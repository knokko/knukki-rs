use crate::{HorizontalTextAlignment, VerticalTextAlignment};

#[derive(Copy, Clone, Debug)]
pub struct DrawnTextPosition {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

#[derive(Copy, Clone, Debug)]
pub struct TextDrawPosition {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
    pub horizontal_alignment: HorizontalTextAlignment,
    pub vertical_alignment: VerticalTextAlignment,
}