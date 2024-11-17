use crate::sprite::utils::color_key_util::ColorKey;
use crate::texture::Texture;
use glam::Vec2;

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(
        width: f32,
        height: f32,
    ) -> Self {
        Self { width, height }
    }
}

#[derive(Debug, Clone)]
pub struct Sprite {
    pub texture: Texture,
    pub position: Vec2,
    pub rotation: f32,
    pub tex_coords: [Vec2; 4],
    pub color_key: Option<ColorKey>,
    pub sprite_size: Rect,
    pixel_scale: f32,
    current_frame: usize,
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
        color_key: Option<&str>,
    ) -> Self {
        let color_key = if let Some(hex) = color_key {
            Some(ColorKey::from_hex(hex, 0.01).unwrap())
        } else {
            None
        };

        Self {
            texture,
            position,
            rotation,
            current_frame: 0,
            tex_coords: Self::get_frame_coords(
                0,
                sprite_size.clone(),
                sheet_size.clone(),
            ),
            sprite_size,
            sheet_size,
            pixel_scale,
            color_key,
        }
    }

    /// Set color key from hex string
    ///
    /// # Arguments
    /// * `hex` - Hex color code (e.g. "#FFFFFF" or "FFFFFF")
    ///
    /// # Returns
    /// * `Result<(), String>` - Result of operation
    ///
    /// # Examples
    /// ```
    /// sprite.set_color_key_hex("#FFFFFF").unwrap();
    /// ```
    #[allow(unused)]
    pub fn set_color_key_hex(
        &mut self,
        hex: &str,
    ) -> Result<(), String> {
        self.color_key = Some(ColorKey::from_hex(hex, 0.5)?);
        Ok(())
    }

    /// Set color key threshold
    ///
    /// # Arguments
    /// * `threshold` - Threshold value
    ///
    /// # Returns
    /// * `Result<(), String>` - Result of operation
    ///
    /// # Examples
    /// ```
    /// sprite.set_color_key_threshold(0.5).unwrap();
    /// ```
    #[allow(unused)]
    pub fn set_color_key_threshold(
        &mut self,
        threshold: f32,
    ) -> Result<(), String> {
        if let Some(ref mut color_key) = self.color_key {
            color_key.threshold = threshold;
        }
        Ok(())
    }

    pub fn get_current_frame(&self) -> usize {
        self.current_frame
    }

    pub fn get_pixel_scale(&self) -> f32 {
        self.pixel_scale
    }

    pub fn get_frame_coords(
        frame: usize,
        sprite_size: Rect,
        sheet_size: Rect,
    ) -> [Vec2; 4] {
        let frames_per_row = (sheet_size.width / sprite_size.width) as usize;
        let frames_per_col = (sheet_size.height / sprite_size.height) as usize;

        // Check if the frame is within the valid range
        let total_frames = frames_per_row * frames_per_col;
        let frame = frame % total_frames;

        let grid_row = frame / frames_per_row;
        let grid_col = frame % frames_per_row;

        // Calculate UV coordinates
        let u = (grid_col as f32 * sprite_size.width) / sheet_size.width;
        let v = (grid_row as f32 * sprite_size.height) / sheet_size.height;
        let u2 = u + (sprite_size.width / sheet_size.width);
        let v2 = v + (sprite_size.height / sheet_size.height);

        [
            Vec2::new(u, v2),  // Left bottom
            Vec2::new(u2, v2), // Right bottom
            Vec2::new(u2, v),  // Right top
            Vec2::new(u, v),   // Left top
        ]
    }

    pub fn update_frame(
        &mut self,
        frame: usize,
    ) {
        self.current_frame = frame;
        self.tex_coords = Self::get_frame_coords(
            frame,
            self.sprite_size.clone(),
            self.sheet_size.clone(),
        );
    }
}
