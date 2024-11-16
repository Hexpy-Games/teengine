use glam::{Vec2, Vec3};
use crate::texture::Texture;

#[derive(Debug, Clone, Copy)]
pub struct ColorKey {
    pub color: Vec3,
    pub threshold: f32,
}

impl ColorKey {
    /// hex 코드로부터 ColorKey 생성 (예: "#FF00FF" 또는 "FF00FF")
    pub fn from_hex(hex: &str, threshold: f32) -> Result<Self, String> {
        let hex = hex.trim_start_matches('#');
        
        if hex.len() != 6 {
            return Err("Invalid hex color code length".to_string());
        }

        // hex 문자열을 u8 바이트로 변환
        let r = u8::from_str_radix(&hex[0..2], 16)
            .map_err(|_| "Invalid red component")?;
        let g = u8::from_str_radix(&hex[2..4], 16)
            .map_err(|_| "Invalid green component")?;
        let b = u8::from_str_radix(&hex[4..6], 16)
            .map_err(|_| "Invalid blue component")?;
        
        Ok(Self {
            color: Vec3::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0),
            threshold
        })
    }

    /// RGB 값으로부터 ColorKey 생성 (0-255 범위)
    pub fn from_rgb(r: u8, g: u8, b: u8, threshold: f32) -> Self {
        Self {
            color: Vec3::new(
                r as f32 / 255.0,
                g as f32 / 255.0,
                b as f32 / 255.0
            ),
            threshold
        }
    }

    /// hex 문자열 반환
    pub fn to_hex(&self) -> String {
        let r = (self.color.x * 255.0) as u8;
        let g = (self.color.y * 255.0) as u8;
        let b = (self.color.z * 255.0) as u8;
        format!("#{:02X}{:02X}{:02X}", r, g, b)
    }
}