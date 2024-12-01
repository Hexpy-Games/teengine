use glam::Vec2;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::Texture;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TilesetData {
    pub name: String,
    pub image_path: String,
    pub tile_width: u32,
    pub tile_height: u32,
    pub spacing: u32,
    pub margin: u32,
    pub tiles: HashMap<u32, TileData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileData {
    pub id: u32,
    pub properties: TileProperties,
    pub animation: Option<AnimationData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationData {
    pub frames: Vec<AnimationFrame>,
    pub duration: f32, // total animation duration (seconds)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationFrame {
    pub tile_id: u32,
    pub duration: f32, // frame duration (seconds)
}

pub struct Tileset {
    texture: Texture,
    tile_data: TilesetData,
    tile_uvs: HashMap<u32, [Vec2; 4]>, // UV coordinates for each tile
    animated_tiles: HashMap<u32, AnimatedTile>,
}

struct AnimatedTile {
    current_frame: usize,
    elapsed_time: f32,
    animation_data: AnimationData,
}

impl Tileset {
    pub fn new(tileset_path: &Path) -> Result<Self, String> {
        // load data from tileset.json file
        let tileset_file = std::fs::File::open(tileset_path)
            .map_err(|e| format!("Failed to open tileset file: {}", e))?;

        let tile_data: TilesetData = serde_json::from_reader(tileset_file)
            .map_err(|e| format!("Failed to parse tileset data: {}", e))?;

        // load texture
        let texture_path = Path::new(&tile_data.image_path);
        let texture = Texture::new(texture_path)?;

        // calculate UV coordinates
        let mut tile_uvs = HashMap::new();
        let mut animated_tiles = HashMap::new();

        Self::calculate_tile_uvs(
            &tile_data,
            &mut tile_uvs,
            &mut animated_tiles,
            texture.width(),
            texture.height(),
        );

        Ok(Self {
            texture,
            tile_data,
            tile_uvs,
            animated_tiles,
        })
    }

    fn calculate_tile_uvs(
        tile_data: &TilesetData,
        tile_uvs: &mut HashMap<u32, [Vec2; 4]>,
        animated_tiles: &mut HashMap<u32, AnimatedTile>,
        texture_width: u32,
        texture_height: u32,
    ) {
        let cols = (texture_width - tile_data.margin * 2 + tile_data.spacing)
            / (tile_data.tile_width + tile_data.spacing);

        for (id, data) in &tile_data.tiles {
            // calculate tile row and column
            let tile_col = id % cols;
            let tile_row = id / cols;

            // calculate tile position in texture
            let x = tile_data.margin
                + tile_col * (tile_data.tile_width + tile_data.spacing);
            let y = tile_data.margin
                + tile_row * (tile_data.tile_height + tile_data.spacing);

            // calculate UV coordinates (normalized to texture coordinates)
            let u1 = x as f32 / texture_width as f32;
            let v1 = y as f32 / texture_height as f32;
            let u2 = (x + tile_data.tile_width) as f32 / texture_width as f32;
            let v2 = (y + tile_data.tile_height) as f32 / texture_height as f32;

            // store UV coordinates [bottom left, bottom right, top right, top left]
            tile_uvs.insert(
                *id,
                [
                    Vec2::new(u1, v2),
                    Vec2::new(u2, v2),
                    Vec2::new(u2, v1),
                    Vec2::new(u1, v1),
                ],
            );

            // if there is an animation, create AnimatedTile
            if let Some(anim_data) = &data.animation {
                animated_tiles.insert(
                    *id,
                    AnimatedTile {
                        current_frame: 0,
                        elapsed_time: 0.0,
                        animation_data: anim_data.clone(),
                    },
                );
            }
        }
    }

    pub fn get_tile_uvs(
        &self,
        tile_id: u32,
    ) -> Option<&[Vec2; 4]> {
        self.tile_uvs.get(&tile_id)
    }

    pub fn get_tile_properties(
        &self,
        tile_id: u32,
    ) -> Option<&TileProperties> {
        self.tile_data
            .tiles
            .get(&tile_id)
            .map(|data| &data.properties)
    }

    pub fn update_animations(
        &mut self,
        delta_time: f32,
    ) {
        for animated_tile in self.animated_tiles.values_mut() {
            animated_tile.elapsed_time += delta_time;

            let current_frame = &animated_tile.animation_data.frames
                [animated_tile.current_frame];

            // check if current frame duration has passed
            if animated_tile.elapsed_time >= current_frame.duration {
                animated_tile.current_frame = (animated_tile.current_frame + 1)
                    % animated_tile.animation_data.frames.len();
                animated_tile.elapsed_time = 0.0;
            }
        }
    }

    pub fn get_current_frame_id(
        &self,
        tile_id: u32,
    ) -> u32 {
        if let Some(animated_tile) = self.animated_tiles.get(&tile_id) {
            animated_tile.animation_data.frames[animated_tile.current_frame]
                .tile_id
        } else {
            tile_id
        }
    }

    pub fn bind_texture(&self) {
        self.texture.bind();
    }
}
