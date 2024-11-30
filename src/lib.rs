// teengine/src/lib.rs
pub use gl;

pub mod core;
pub mod input;
pub mod sprite;
pub mod texture;

pub use core::Engine;
pub use core::Game;
pub use input::input_manager::InputAction;
pub use input::input_manager::InputManager;
pub use sprite::animation::AnimationProvider;
pub use sprite::animation::AnimationSequence;
pub use sprite::animation_sprite::AnimatedSprite;
pub use sprite::sprite::{Rect, Sprite};
pub use sprite::sprite_renderer::SpriteRenderer;
pub use texture::texture::Texture;
