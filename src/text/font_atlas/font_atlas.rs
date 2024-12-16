use image::{Rgba, RgbaImage};
use rusttype::{point, Font, PositionedGlyph, Scale};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};

use super::{FontAtlasFile, FontError};

// Define the character information in the font atlas
#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct CharInfo {
    pub x: f32,        // X position in atlas
    pub y: f32,        // Y position in atlas
    pub width: f32,    // Width of character
    pub height: f32,   // Height of character
    pub xoffset: f32,  // Offset from cursor position
    pub yoffset: f32,  // Offset from cursor position
    pub xadvance: f32, // How far to move cursor for next character
}

impl CharInfo {
    pub fn new(
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        xoffset: f32,
        yoffset: f32,
        xadvance: f32,
    ) -> Self {
        // Normalize values to be in the range [0, 1]
        Self {
            x,
            y,
            width,
            height,
            xoffset,
            yoffset,
            xadvance,
        }
    }
}

#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
pub enum FontType {
    Ascii,
    Unicode,
}

impl FontType {
    pub fn from_str(s: &str) -> Result<Self, FontError> {
        match s.to_uppercase().as_str() {
            "ASCII" => Ok(FontType::Ascii),
            "UNICODE" => Ok(FontType::Unicode),
            invalid_type => {
                Err(FontError::InvalidFontType(invalid_type.to_string()))
            }
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            FontType::Ascii => "ASCII",
            FontType::Unicode => "UNICODE",
        }
    }
}

#[derive(Debug)]
pub struct FontAtlas {
    pub image: RgbaImage,
    pub chars: HashMap<char, CharInfo>,
    pub font_size: f32,
    pub padding: u32,
    pub width: u32,
    pub height: u32,
    pub line_height: f32,
    pub font_type: FontType,
}

const DEFAULT_PADDING: u32 = 10;
const DEFAULT_FONT_SIZE: f32 = 16.0;
const DEFAULT_FONT_TYPE: FontType = FontType::Unicode;

impl FontAtlas {
    pub const CURRENT_VERSION: u32 = 1;

    pub fn new(
        font_data: &[u8],
        font_size: Option<f32>,
        font_type: Option<String>,
        padding: Option<u32>,
    ) -> Result<Self, FontError> {
        let font_type = if let Some(ft_str) = font_type {
            FontType::from_str(&ft_str)?
        } else {
            DEFAULT_FONT_TYPE
        };

        let baked_font_atlas =
            Self::bake_font_atlas(font_data, font_size, font_type, padding)
                .map_err(|e| FontError::FontLoadError(e.to_string()))?;

        Ok(Self {
            image: baked_font_atlas.image,
            chars: baked_font_atlas.chars,
            padding: baked_font_atlas.padding,
            width: baked_font_atlas.width,
            height: baked_font_atlas.height,
            line_height: baked_font_atlas.line_height,
            font_type: baked_font_atlas.font_type,
            font_size: baked_font_atlas.font_size,
        })
    }

    // Utility function to bake font into an atlas
    pub fn bake_font_atlas(
        font_data: &[u8],
        font_size: Option<f32>,
        font_type: FontType,
        padding: Option<u32>,
    ) -> Result<FontAtlas, Box<dyn std::error::Error + '_>> {
        // Load font from font_data
        let font = Font::try_from_bytes(font_data).ok_or_else(|| {
            FontError::FontLoadError("Failed to load font".to_string())
        })?;

        let font_size = font_size.unwrap_or(DEFAULT_FONT_SIZE);
        let scale = Scale::uniform(font_size);
        let v_metrics = font.v_metrics(scale);
        let line_height =
            v_metrics.ascent - v_metrics.descent + v_metrics.line_gap;

        let chars: Vec<char>;

        if font_type == FontType::Unicode {
            chars = (0..=0xFFFF) // Basic Multilingual Plane (BMP)
                .filter(|&c| {
                    font.glyph(char::from_u32(c).unwrap_or(' ')).id().0 > 0
                }) // Filter valid glyphs
                .filter(|&c| char::from_u32(c) != None)
                .map(|c| char::from_u32(c).unwrap())
                .collect();
        } else {
            chars = (32..127).map(|c| c as u8 as char).collect(); // ASCII printable characters
        }

        // First pass: calculate total width and height needed
        let mut total_width: u32 = 0;
        let mut max_height: u32 = 0;
        let glyphs: Vec<_> =
            chars.iter().map(|c| font.glyph(*c).scaled(scale)).collect();

        let padding = padding.unwrap_or(DEFAULT_PADDING);

        for glyph in &glyphs {
            if let Some(bbox) = glyph.exact_bounding_box() {
                total_width += bbox.width() as u32 + padding;
                max_height = max_height.max(bbox.height() as u32 + padding);
            }
        }

        // Calculate rows and columns for a more square-like atlas
        let approx_sqrt = (total_width as f32).sqrt().ceil() as u32;
        let atlas_width = approx_sqrt.next_power_of_two();
        let num_rows =
            ((total_width as f32) / (atlas_width as f32)).ceil() as u32;
        let atlas_height = (max_height * num_rows).next_power_of_two();

        // Create the atlas image
        let mut atlas = RgbaImage::new(atlas_width, atlas_height);
        let mut char_info = HashMap::new();
        let mut cursor_x = 0;
        let mut cursor_y = 0;

        // Second pass: render glyphs to atlas
        for (c, glyph) in chars.iter().zip(glyphs.iter()) {
            if let Some(bbox) = glyph.exact_bounding_box() {
                // Check if we need to move to next row
                if cursor_x + bbox.width() as u32 + padding > atlas_width {
                    cursor_x = 0;
                    cursor_y += max_height;
                }

                let glyph = glyph.clone().positioned(point(0.0, 0.0));

                // Draw glyph into atlas
                Self::draw_glyph_to_atlas(
                    &mut atlas, &glyph, cursor_x, cursor_y,
                );

                // Store character information
                char_info.insert(
                    *c,
                    CharInfo {
                        x: cursor_x as f32 / atlas_width as f32,
                        y: cursor_y as f32 / atlas_height as f32,
                        width: bbox.width() as f32 / atlas_width as f32,
                        height: bbox.height() as f32 / atlas_height as f32,
                        xoffset: bbox.min.x as f32,
                        yoffset: -bbox.min.y as f32,
                        xadvance: glyph
                            .unpositioned()
                            .h_metrics()
                            .advance_width,
                    },
                );

                cursor_x += bbox.width() as u32 + padding;
            }
        }

        Ok(FontAtlas {
            image: atlas,
            chars: char_info,
            padding,
            width: atlas_width,
            height: atlas_height,
            line_height,
            font_type,
            font_size,
        })
    }

    fn draw_glyph_to_atlas(
        atlas: &mut RgbaImage,
        glyph: &PositionedGlyph,
        x: u32,
        y: u32,
    ) {
        if let Some(bb) = glyph.pixel_bounding_box() {
            for px in x..(x + bb.width() as u32) {
                for py in y..(y + bb.height() as u32) {
                    atlas.put_pixel(px, py, Rgba([255, 255, 255, 0]));
                }
            }

            glyph.draw(|gx, gy, intensity| {
                let px = x + gx as u32;
                let py = y + gy as u32;

                let alpha = (intensity * 255.0) as u8;
                let rgba = Rgba([255, 255, 255, alpha]);
                atlas.put_pixel(px, py, rgba);
            });
        }
    }

    pub fn save_to_file(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<(), FontError> {
        FontAtlasFile::save_to_file(self, path)
    }

    pub fn load_from_file(
        path: impl AsRef<Path>
    ) -> Result<FontAtlas, FontError> {
        FontAtlasFile::load_from_file(path)
    }

    pub fn default() -> Result<Self, FontError> {
        static DEFAULT_FONT_ATLAS: &[u8] =
            include_bytes!("../../assets/fonts/pretendard.fad");
        FontAtlasFile::load_from_bytes(DEFAULT_FONT_ATLAS)
    }
}
