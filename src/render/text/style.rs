use crate::*;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct TextStyle {
    pub font_id: Option<String>,
    pub text_color: Color,
    pub background_color: Color,
    pub background_fill_mode: TextBackgroundFillMode
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum TextBackgroundFillMode {
    DoNot,
    DrawnRegion,
    EntireDomain
}