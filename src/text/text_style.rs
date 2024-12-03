use std::ops::Range;

#[derive(Clone, Debug)]
pub struct TextStyle {
    pub font_size: f32,
    pub color: [f32; 4],
    pub line_height: f32,
    pub letter_spacing: f32,
    pub alignment: TextAlignment,
    pub max_width: Option<f32>,
    pub vertical_alignment: VerticalAlignment,
    pub background_color: Option<[f32; 4]>,
    pub padding: Padding,
    pub clip_bounds: Option<Range<f32>>,
}

#[derive(Clone, Debug)]
pub struct Padding {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
}

#[derive(Clone, Debug, PartialEq)]
pub enum VerticalAlignment {
    Top,
    Middle,
    Bottom,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_size: 16.0,
            color: [1.0, 1.0, 1.0, 1.0],
            line_height: 1.2,
            letter_spacing: 0.0,
            alignment: TextAlignment::Left,
            max_width: None,
            vertical_alignment: VerticalAlignment::Top,
            background_color: None,
            padding: Padding::default(),
            clip_bounds: None,
        }
    }
}

impl Default for Padding {
    fn default() -> Self {
        Self {
            left: 0.0,
            right: 0.0,
            top: 0.0,
            bottom: 0.0,
        }
    }
}
