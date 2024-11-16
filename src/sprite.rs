use crate::texture::Texture;
use glam::Vec2;

#[derive(Clone)]
pub struct Rect {
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

pub struct Sprite {
    pub texture: Texture,
    pub position: Vec2,
    pub rotation: f32,
    pub tex_coords: [Vec2; 4],
    pixel_scale: f32,
    current_frame: usize,
    frames_per_row: usize,      // 한 행의 프레임 수
    frames_per_column: usize,   // 한 열의 프레임 수
    sprite_size: Rect,
    sheet_size: Rect,
}

impl Sprite {
    pub fn new(
        texture: Texture,
        position: Vec2,
        rotation: f32,
        sprite_size: Rect,
        sheet_size: Rect,
        pixel_scale: f32,
    ) -> Self {
        let frames_per_row = (sheet_size.width / sprite_size.width) as usize;
        let frames_per_column = (sheet_size.height / sprite_size.height) as usize;
        
        Self {
            texture,
            position,
            rotation,
            current_frame: 0,
            tex_coords: Self::get_frame_coords(0, sprite_size.clone(), sheet_size.clone()),
            frames_per_row,
            frames_per_column,
            sprite_size,
            sheet_size,
            pixel_scale,
        }
    }

    pub fn get_current_frame(&self) -> usize {
        self.current_frame
    }

    pub fn get_frame_dimensions(&self) -> Vec2 {
        Vec2::new(self.frames_per_row as f32, self.frames_per_column as f32)
    }

    pub fn get_pixel_scale(&self) -> f32 {
        self.pixel_scale
    }

    pub fn get_frame_coords(frame: usize, sprite_size: Rect, sheet_size: Rect) -> [Vec2; 4] {
        let frames_per_row = (sheet_size.width / sprite_size.width) as usize;

        let grid_row = frame / frames_per_row;
        let grid_col = frame % frames_per_row;
        
        let s = (grid_col as f32 * sprite_size.width) / sheet_size.width;
        let t = (grid_row as f32 * sprite_size.height) / sheet_size.height;
        let s2 = ((grid_col as f32 * sprite_size.width) + sprite_size.width ) / sheet_size.width;
        let t2 = ((grid_row as f32 * sprite_size.height) + sprite_size.height) / sheet_size.height;

        [
            Vec2::new(s, t2), // bottom left
            Vec2::new(s2, t2), // bottom right
            Vec2::new(s2, t), // top right
            Vec2::new(s, t), // top left
        ]
    }

    pub fn update_frame(&mut self, frame: usize) {
        self.current_frame = frame;
        self.tex_coords = Self::get_frame_coords(
            frame, 
            self.sprite_size.clone(), 
            self.sheet_size.clone(),
        );
    }
}
