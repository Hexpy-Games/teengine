use glam::Vec2;
use rand::prelude::*;
use std::path::Path;
use std::time::{Duration, Instant};
use teengine::{
    input::input_manager::InputAction,
    tile::{
        TileInstance, TileLayer, TileMap, TileMapRenderer, TileProperties,
        Tileset,
    },
    AnimatedSprite, AnimationSequence, Engine, Game, Rect, Sprite, Texture,
};

struct PlayerAnimations {
    idle: AnimationSequence,
    walking: AnimationSequence,
}

impl PlayerAnimations {
    fn new() -> Self {
        Self {
            idle: AnimationSequence::new(
                "idle".to_string(),
                vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
                100,
                true,
            ),
            walking: AnimationSequence::new(
                "walking".to_string(),
                vec![16, 17, 0, 18, 19, 0],
                100,
                true,
            ),
        }
    }
}

#[derive(Clone, PartialEq)]
enum AnimationState {
    Idle,
    Walking,
}

struct SimpleGame {
    sprite: Option<Sprite>,
    animated_sprite: Option<AnimatedSprite>,
    player_animations: PlayerAnimations,
    last_frame_change: Instant,
    frame_duration: Duration,
    current_animation_state: AnimationState,
    tilemap: Option<TileMap>,
    tilemap_renderer: Option<TileMapRenderer>,
}

impl SimpleGame {
    pub fn new() -> Self {
        Self {
            sprite: None,
            animated_sprite: None,
            player_animations: PlayerAnimations::new(),
            last_frame_change: Instant::now(),
            frame_duration: Duration::from_millis(120),
            current_animation_state: AnimationState::Idle,
            tilemap: None,
            tilemap_renderer: None,
        }
    }

    fn init_tilemap(&mut self) -> Result<(), String> {
        // load tileset
        let tileset = Tileset::new(Path::new("assets/tileset.json")).unwrap();

        // create 20x15 tilemap (640x480 pixels)
        let mut tilemap = TileMap::new(20, 15, 32, 4.0, tileset);

        // create ground layer
        let mut ground_layer = TileLayer::new(20, 15);

        for y in 0..15 {
            for x in 0..20 {
                let tile = TileInstance {
                    id: 0, // grass tile id
                    properties: tilemap
                        .tileset
                        .get_tile_properties(0)
                        .unwrap_or(&TileProperties::new_default())
                        .clone(),
                };
                ground_layer.set_tile(x as u32, y as u32, tile);
            }
        }

        // Generate a random box tiles
        for _ in 0..10 {
            let x = rand::thread_rng().gen_range(0..20);
            let y = rand::thread_rng().gen_range(0..15);
            let tile = TileInstance {
                id: 2,
                properties: tilemap
                    .tileset
                    .get_tile_properties(2)
                    .unwrap_or(&TileProperties::new_default())
                    .clone(),
            };
            ground_layer.set_tile(x as u32, y as u32, tile);
        }

        tilemap.add_layer("ground".to_string(), ground_layer);
        self.tilemap = Some(tilemap);

        Ok(())
    }

    fn check_collision(
        &self,
        pos: Vec2,
    ) -> bool {
        if let Some(tilemap) = &self.tilemap {
            let (tile_x, tile_y) = tilemap.world_to_tile(pos);

            // Calculate the tile range to check for collisions
            let check_width = 1; // adjust according to the character size
            let check_height = 1;

            // check collisions with nearby tiles
            for y in tile_y.saturating_sub(check_height)..=tile_y + check_height
            {
                for x in
                    tile_x.saturating_sub(check_width)..=tile_x + check_width
                {
                    if let Some(tile) = tilemap.get_tile_at("ground", x, y) {
                        if tile.properties.is_collidable() {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
}

impl Game for SimpleGame {
    fn init(
        &mut self,
        _engine: &Engine,
    ) {
        let tileset = Tileset::new(Path::new("assets/tileset.json"));

        let texture = Texture::new(Path::new("assets/sprite.png"))
            .expect("Failed to load texture");

        let sprite = Sprite::new(
            texture,
            Vec2::new(100.0, 300.0),
            0.0,
            Rect::new(18.0, 18.0),
            Rect::new(90.0, 90.0),
            4.0,
            None,
        );

        self.animated_sprite = Some(AnimatedSprite::new(sprite.clone()));
        self.sprite = Some(sprite);

        if let Some(animated_sprite) = &mut self.animated_sprite {
            animated_sprite
                .set_animation(Box::new(self.player_animations.idle.clone()));
        }

        if let Err(e) = self.init_tilemap() {
            eprintln!("Failed to initialize tilemap: {}", e);
        }

        self.tilemap_renderer = Some(
            TileMapRenderer::new(1000)
                .expect("Failed to create tilemap renderer"),
        );
    }

    fn update(
        &mut self,
        engine: &mut Engine,
    ) {
        let mut is_moving = false;
        let movement_speed = 2.0;

        if let Some(animated_sprite) = &mut self.animated_sprite {
            let mut new_pos = animated_sprite.sprite().position;
            let weight = movement_speed;

            if engine
                .input_manager
                .is_action_active(InputAction::MoveRight)
            {
                new_pos.x += weight;
                is_moving = true;
            }
            if engine.input_manager.is_action_active(InputAction::MoveLeft) {
                new_pos.x -= weight;
                is_moving = true;
            }
            if engine.input_manager.is_action_active(InputAction::MoveUp) {
                new_pos.y -= weight;
                is_moving = true;
            }
            if engine.input_manager.is_action_active(InputAction::MoveDown) {
                new_pos.y += weight;
                is_moving = true;
            }

            animated_sprite.sprite_mut().position = new_pos;

            let new_state = if is_moving {
                AnimationState::Walking
            } else {
                AnimationState::Idle
            };

            // set new animation only when the state changes
            if self.current_animation_state != new_state {
                self.current_animation_state = new_state.clone();
                match new_state {
                    AnimationState::Walking => {
                        animated_sprite.set_animation(Box::new(
                            self.player_animations.walking.clone(),
                        ));
                    }
                    AnimationState::Idle => {
                        animated_sprite.set_animation(Box::new(
                            self.player_animations.idle.clone(),
                        ));
                    }
                }
            }

            // update animation
            if self.last_frame_change.elapsed() >= self.frame_duration {
                animated_sprite.update();
                self.last_frame_change = Instant::now();
            }
        }

        // update tilemap
        if let Some(tilemap) = &mut self.tilemap {
            tilemap.update(engine.delta_time());
        }
    }

    fn render(
        &mut self,
        engine: &Engine,
    ) {
        if let Some(tilemap) = &self.tilemap {
            if let Some(renderer) = &mut self.tilemap_renderer {
                renderer.render(tilemap, &engine.projection);
            }
        }

        if let Some(animated_sprite) = &self.animated_sprite {
            engine
                .sprite_renderer
                .draw_sprite(animated_sprite.sprite(), &engine.projection);
        }
    }
}

fn main() {
    let game = SimpleGame::new();
    Engine::new_with_game("Simple Game Example", 800, 600, true, game);
}
