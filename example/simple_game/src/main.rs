use glam::Vec2;
use rand::prelude::*;
use std::path::Path;
use std::time::{Duration, Instant};
use teengine::core::EngineConfig;
use teengine::{
    input::input_manager::InputAction,
    tile::{
        TileInstance, TileLayer, TileMap, TileMapRenderer, TileProperties,
        Tileset,
    },
    AnimatedSprite, AnimationSequence, Engine, Game, Rect, Sprite, Texture,
};
use teengine::{FontAtlas, TextRenderer};

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
    text_renderer: Option<TextRenderer>,
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
            text_renderer: None,
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
        tilemap: Option<&TileMap>,
        pos: Vec2,
        world_bounds: Option<(Vec2, Vec2)>,
    ) -> bool {
        // check world bounderies
        if let Some(world_bounds) = world_bounds {
            if pos.x < world_bounds.0.x
                || pos.x > (world_bounds.1.x - 72.0)
                || pos.y < world_bounds.0.y
                || pos.y > (world_bounds.1.y - 72.0)
            {
                return true;
            }
        }

        if let Some(tilemap) = tilemap {
            let tile_size = tilemap.tile_size as f32 * tilemap.scale;
            let character_width = 18.0 * 4.0;
            let character_height = 18.0 * 4.0;

            let char_left = pos.x;
            let char_right = pos.x + character_width;
            let char_top = pos.y;
            let char_bottom = pos.y + character_height;

            let start_x = (char_left / tile_size).floor() as i32;
            let end_x = (char_right / tile_size).ceil() as i32;
            let start_y = (char_top / tile_size).floor() as i32;
            let end_y = (char_bottom / tile_size).ceil() as i32;

            for y in start_y..=end_y {
                for x in start_x..=end_x {
                    if let Some(tile) =
                        tilemap.get_tile_at("ground", x as u32, y as u32)
                    {
                        if tile.properties.is_collidable() {
                            let tile_left = x as f32 * tile_size;
                            let tile_right = tile_left + tile_size;
                            let tile_top = y as f32 * tile_size;
                            let tile_bottom = tile_top + tile_size;

                            if char_left < tile_right
                                && char_right > tile_left
                                && char_top < tile_bottom
                                && char_bottom > tile_top
                            {
                                return true;
                            }
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
        engine: &mut Engine,
    ) {
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

        let camera = &mut engine.camera;

        camera.set_lerp_speed(1.0);

        if let Some(tilemap) = &self.tilemap {
            let world_width =
                tilemap.width as f32 * tilemap.tile_size as f32 * tilemap.scale;
            let world_height = tilemap.height as f32
                * tilemap.tile_size as f32
                * tilemap.scale;
            camera.set_world_bounds(
                Vec2::ZERO,
                Vec2::new(world_width, world_height),
            );
        }

        let new_atlas = FontAtlas::new(
            include_bytes!("../assets/Pretendard-Medium.ttf"),
            Some(16.0),
            None,
            None,
        )
        .expect("Failed to create font atlas");
        new_atlas.save_to_file("assets/pretendard.fad").unwrap();

        // let new_atlas =
        //     FontAtlas::default().expect("Failed to create font atlas");

        new_atlas
            .image
            .save("test.png")
            .expect("Failed to save image");

        self.text_renderer = Some(
            TextRenderer::builder()
                .with_font_atlas(new_atlas)
                .build()
                .expect("Failed to create text renderer"),
        );
    }

    fn update(
        &mut self,
        engine: &mut Engine,
    ) {
        let mut is_moving = false;
        let base_movement_speed = 300.0; // move 300 pixels per second
        let tilemap = self.tilemap.as_ref();

        if let Some(animated_sprite) = &mut self.animated_sprite {
            let current_pos = animated_sprite.sprite().position.clone();
            let mut new_pos = animated_sprite.sprite().position.clone();

            // delta time을 곱해서 프레임 레이트와 무관하게 일정한 속도 유지
            let move_weight = base_movement_speed * engine.delta_time();

            if engine
                .input_manager
                .is_action_active(InputAction::MoveRight)
            {
                new_pos.x += move_weight;
                is_moving = true;
            }
            if engine.input_manager.is_action_active(InputAction::MoveLeft) {
                new_pos.x -= move_weight;
                is_moving = true;
            }
            if engine.input_manager.is_action_active(InputAction::MoveUp) {
                new_pos.y -= move_weight;
                is_moving = true;
            }
            if engine.input_manager.is_action_active(InputAction::MoveDown) {
                new_pos.y += move_weight;
                is_moving = true;
            }

            if Self::check_collision(
                tilemap,
                new_pos,
                engine.camera.get_world_bounds(),
            ) {
                new_pos = current_pos;
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

        if let Some(animated_sprite) = &self.animated_sprite {
            let player_pos = animated_sprite.sprite().position;
            let player_center_pos = player_pos + Vec2::new(18.0, 18.0);
            engine
                .camera
                .follow_target(player_center_pos, engine.delta_time());
        }
    }

    fn render(
        &mut self,
        engine: &mut Engine,
    ) {
        let projection = engine.camera.get_projection_matrix();

        if let Some(tilemap) = &self.tilemap {
            if let Some(renderer) = &mut self.tilemap_renderer {
                renderer.render(tilemap, &projection);
            }
        }

        if let Some(animated_sprite) = &self.animated_sprite {
            engine
                .sprite_renderer
                .draw_sprite(animated_sprite.sprite(), &projection);

            if let Some(text_renderer) = &mut self.text_renderer {
                let pos = animated_sprite.sprite().position;
                let text = format!("Coord X: {:.1}, Y: {:.1}", pos.x, pos.y);
                // let text = format!("Hello world!");

                //     let style = TextStyle {
                //         font_size: 26.0,
                //         color: [1.0, 1.0, 1.0, 1.0],
                //         alignment: TextAlignment::Left,
                //         background_color: Some([0.0, 0.0, 0.0, 0.7]),
                //         padding: Padding {
                //             left: 5.0,
                //             right: 5.0,
                //             top: 3.0,
                //             bottom: 3.0,
                //         },
                //         ..Default::default()
                //     };

                // UI 좌표는 카메라와 독립적이어야 하므로 engine.projection 사용
                text_renderer.render_text(
                    &text,
                    40.0,
                    40.0,
                    2.0,
                    [1.0, 1.0, 1.0, 1.0],
                    &engine.projection,
                );
            }
        }
    }
}

fn main() {
    let config = EngineConfig {
        title: "Simple Game Example".to_string(),
        width: 800,
        height: 600,
        vsync: true,
        fps_limit: None,
    };

    let game = SimpleGame::new();
    Engine::new_with_game(config, game);
}
