use super::{animation::AnimationProvider, sprite::Sprite};

pub struct AnimatedSprite {
    sprite: Sprite,
    animation_provider: Option<Box<dyn AnimationProvider>>,
}

impl AnimatedSprite {
    pub fn new(sprite: Sprite) -> Self {
        Self {
            sprite,
            animation_provider: None,
        }
    }

    pub fn set_animation(
        &mut self,
        provider: Box<dyn AnimationProvider>,
    ) {
        self.animation_provider = Some(provider);
    }

    pub fn clear_animation(&mut self) {
        self.animation_provider = None;
    }

    pub fn update(&mut self) {
        if let Some(provider) = &mut self.animation_provider {
            if let Some(frame) = provider.get_current_frame() {
                self.sprite.update_frame(frame);
            }
        }
    }

    pub fn sprite(&self) -> &Sprite {
        &self.sprite
    }

    pub fn sprite_mut(&mut self) -> &mut Sprite {
        &mut self.sprite
    }
}
