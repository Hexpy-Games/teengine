use glam::Vec2;
use std::collections::HashMap;

use super::{TileProperties, Tileset};

#[derive(Debug)]
pub struct TileMap {
    pub width: u32,
    pub height: u32,
    pub tile_size: u32,
    pub scale: f32,
    pub tileset: Tileset,
    pub layers: HashMap<String, TileLayer>,
}

#[derive(Debug)]
pub struct TileLayer {
    pub visible: bool,
    pub tiles: Vec<Vec<Option<TileInstance>>>,
}

#[derive(Debug, Clone)]
pub struct TileInstance {
    pub id: u32,
    pub properties: TileProperties,
}

impl TileMap {
    pub fn new(
        width: u32,
        height: u32,
        tile_size: u32,
        scale: f32,
        tileset: Tileset,
    ) -> Self {
        Self {
            width,
            height,
            tile_size,
            scale,
            layers: HashMap::new(),
            tileset,
        }
    }

    pub fn add_layer(
        &mut self,
        name: String,
        layer: TileLayer,
    ) {
        self.layers.insert(name, layer);
    }

    pub fn get_tile_at(
        &self,
        layer_name: &str,
        x: u32,
        y: u32,
    ) -> Option<&TileInstance> {
        self.layers
            .get(layer_name)
            .and_then(|layer| layer.tiles.get(y as usize))
            .and_then(|row| row.get(x as usize))
            .and_then(|tile| tile.as_ref())
    }

    pub fn world_to_tile(
        &self,
        world_pos: Vec2,
    ) -> (u32, u32) {
        (
            (world_pos.x / (self.tile_size as f32 * self.scale)) as u32,
            (world_pos.y / (self.tile_size as f32 * self.scale)) as u32,
        )
    }

    pub fn tile_to_world(
        &self,
        tile_x: u32,
        tile_y: u32,
    ) -> Vec2 {
        Vec2::new(
            tile_x as f32 * self.tile_size as f32 * self.scale,
            tile_y as f32 * self.tile_size as f32 * self.scale,
        )
    }

    pub fn update(
        &mut self,
        delta_time: f32,
    ) {
        self.tileset.update_animations(delta_time);
    }
}

impl TileLayer {
    pub fn new(
        width: u32,
        height: u32,
    ) -> Self {
        let tiles = vec![vec![None; width as usize]; height as usize];

        Self {
            visible: true,
            tiles,
        }
    }

    pub fn get_tile(
        &self,
        x: u32,
        y: u32,
    ) -> Option<TileInstance> {
        self.tiles.get(y as usize)?.get(x as usize)?.clone()
    }

    pub fn set_tile(
        &mut self,
        x: u32,
        y: u32,
        tile: TileInstance,
    ) {
        if let Some(row) = self.tiles.get_mut(y as usize) {
            if let Some(cell) = row.get_mut(x as usize) {
                *cell = Some(tile);
            }
        }
    }
}
