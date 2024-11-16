use crate::texture::Texture;
use glam::Vec2;

pub struct Sprite {
    pub texture: Texture,
    pub position: Vec2,
    pub scale: Vec2,
    pub rotation: f32,
    pub tex_coords: [Vec2; 4],
    pub current_frame: usize,
    sprite_size: f32,
    sheet_size: f32,
    grid_size: usize, // 한 행/열의 스프라이트 개수
}

impl Sprite {
    pub fn new(
        texture: Texture,
        position: Vec2,
        scale: Vec2,
        rotation: f32,
        sprite_size: f32,
        sheet_size: f32,
        grid_size: usize,
    ) -> Self {
        Self {
            texture,
            position,
            scale,
            rotation,
            tex_coords: Self::get_frame_coords(0, sprite_size, sheet_size, grid_size),
            current_frame: 0,
            sprite_size,
            sheet_size,
            grid_size,
        }
    }

    pub fn get_frame_coords(frame: usize, sprite_size: f32, sheet_size: f32, grid_size: usize) -> [Vec2; 4] {
        let (row, col) = (frame / grid_size, frame % grid_size);
        let normalized_sprite_size = sprite_size / sheet_size;
        
        let (s, t) = (
            col as f32 * normalized_sprite_size,
            row as f32 * normalized_sprite_size,
        );
        let (s2, t2) = (
            s + normalized_sprite_size,
            t + normalized_sprite_size,
        );

        [
            Vec2::new(s, t2),
            Vec2::new(s2, t2),
            Vec2::new(s2, t),
            Vec2::new(s, t),
        ]
    }

    pub fn update_frame(&mut self, frame: usize) {
        self.current_frame = frame;
        self.tex_coords = Self::get_frame_coords(frame, self.sprite_size, self.sheet_size, self.grid_size);
    }
}
