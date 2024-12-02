use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileProperties {
    pub physics: PhysicsProperties,
    pub gameplay: GameplayProperties,
    pub visual: VisualProperties,
    pub custom_properties: HashMap<String, PropertyValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsProperties {
    pub collision: bool,
    pub collision_type: CollisionType,
    pub friction: f32,
    pub restitution: f32,
    pub one_way_platform: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CollisionType {
    None,
    Full,
    Slope(f32),
    Platform,
    Trigger,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameplayProperties {
    pub tile_type: TileType,
    pub damage: f32,
    pub movement_modifier: f32,
    pub interactable: bool,
    pub health: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TileType {
    Ground,
    Water,
    Lava,
    Ice,
    Mud,
    Sand,
    Platform,
    Decoration,
    Trigger,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualProperties {
    pub light_source: bool,
    pub light_color: Option<Color>,
    pub light_intensity: f32,
    pub opacity: f32,
    pub layer: i32,
    pub tint: Option<Color>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PropertyValue {
    Bool(bool),
    Int(i32),
    Float(f32),
    String(String),
}

impl TileProperties {
    pub fn new_default() -> Self {
        Self {
            physics: PhysicsProperties {
                collision: false,
                collision_type: CollisionType::None,
                friction: 0.0,
                restitution: 0.0,
                one_way_platform: false,
            },
            gameplay: GameplayProperties {
                tile_type: TileType::Ground,
                damage: 0.0,
                movement_modifier: 1.0,
                interactable: false,
                health: None,
            },
            visual: VisualProperties {
                light_source: false,
                light_color: None,
                light_intensity: 0.0,
                opacity: 1.0,
                layer: 0,
                tint: None,
            },
            custom_properties: HashMap::new(),
        }
    }

    pub fn with_collision(mut self) -> Self {
        self.physics.collision = true;
        self.physics.collision_type = CollisionType::Full;
        self
    }

    pub fn with_slope(
        mut self,
        angle: f32,
    ) -> Self {
        self.physics.collision = true;
        self.physics.collision_type = CollisionType::Slope(angle);
        self
    }

    pub fn is_collidable(&self) -> bool {
        self.physics.collision
    }
}
