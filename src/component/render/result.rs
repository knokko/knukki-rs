use crate::*;

pub struct RenderResult {
    pub drawn_region: Box<dyn DrawnRegion>,
    pub filter_mouse_actions: bool,
}

impl RenderResult {
    pub fn entire() -> Self {
        Self {
            drawn_region: Box::new(RectangularDrawnRegion::new(0.0, 0.0, 1.0, 1.0)),
            filter_mouse_actions: false,
        }
    }
}
