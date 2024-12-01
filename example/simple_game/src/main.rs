// example/simple_game/src/main.rs
use glam::Vec2;
use std::path::Path;
use std::time::{Duration, Instant};
use teengine::{
    input::input_manager::InputAction, AnimatedSprite, AnimationSequence,
    Engine, Game, Rect, Sprite, Texture,
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
                vec![16, 17, 0, 18, 19],
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
        }
    }
}

impl Game for SimpleGame {
    fn init(
        &mut self,
        _engine: &Engine,
    ) {
        let texture = Texture::new(Path::new("assets/sprite.png"))
            .expect("Failed to load texture");

        let sprite = Sprite::new(
            texture,
            Vec2::new(0.0, 0.0),
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
    }

    fn update(
        &mut self,
        engine: &mut Engine,
    ) {
        let mut is_moving = false;
        let movement_speed = 2.0;

        if let Some(animated_sprite) = &mut self.animated_sprite {
            if engine
                .input_manager
                .is_action_active(InputAction::MoveRight)
            {
                animated_sprite.sprite_mut().position.x += movement_speed;
                is_moving = true;
            }
            if engine.input_manager.is_action_active(InputAction::MoveLeft) {
                animated_sprite.sprite_mut().position.x -= movement_speed;
                is_moving = true;
            }
            if engine.input_manager.is_action_active(InputAction::MoveUp) {
                animated_sprite.sprite_mut().position.y -= movement_speed;
                is_moving = true;
            }
            if engine.input_manager.is_action_active(InputAction::MoveDown) {
                animated_sprite.sprite_mut().position.y += movement_speed;
                is_moving = true;
            }

            let new_state = if is_moving {
                AnimationState::Walking
            } else {
                AnimationState::Idle
            };

            // 상태가 변경될 때만 새 애니메이션 설정
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

            // 애니메이션 업데이트
            if self.last_frame_change.elapsed() >= self.frame_duration {
                animated_sprite.update();
                self.last_frame_change = Instant::now();
            }
        }
    }

    fn render(
        &self,
        engine: &Engine,
    ) {
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
