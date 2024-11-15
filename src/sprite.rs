use crate::texture::Texture;
use glam::Vec2;

pub struct Sprite {
    pub texture: Texture,
    pub position: Vec2,
    pub scale: Vec2,
    pub rotation: f32,
    pub tex_coords: [Vec2; 4],
    pub current_frame: usize,
}

impl Sprite {
    pub fn new(texture: Texture, position: Vec2, scale: Vec2, rotation: f32) -> Self {
        Self {
            texture,
            position,
            scale,
            rotation,
            tex_coords: Self::get_frame_coords(0),
            current_frame: 0,
        }
    }

    pub fn get_frame_coords(frame: usize) -> [Vec2; 4] {
        let sprite_size = 256.0; // 1024 / 4 = 256 (4x4 그리드 가정)
        let (row, col) = (frame / 4, frame % 4);
        let (s, t) = (
            col as f32 * sprite_size / 1024.0,
            row as f32 * sprite_size / 1024.0,
        );
        let (s2, t2) = (
            (col + 1) as f32 * sprite_size / 1024.0,
            (row + 1) as f32 * sprite_size / 1024.0,
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
        self.tex_coords = Self::get_frame_coords(frame);
    }
}
