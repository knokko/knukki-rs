use crate::*;

pub struct RenderResult {

    pub drawn_region: Box<dyn DrawnRegion>,
    pub filter_mouse_actions: bool
}
