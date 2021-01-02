use crate::*;

pub struct RenderResultStruct {
    pub drawn_region: Box<dyn DrawnRegion>,
    pub filter_mouse_actions: bool,
}

#[cfg(feature = "golem_rendering")]
pub type RenderResult = Result<RenderResultStruct, golem::GolemError>;

#[cfg(not(feature = "golem_rendering"))]
pub type RenderResult = Result<RenderResultStruct, ()>;

impl RenderResultStruct {
    pub fn entire() -> Self {
        Self {
            drawn_region: Box::new(RectangularDrawnRegion::new(0.0, 0.0, 1.0, 1.0)),
            filter_mouse_actions: false,
        }
    }
}

pub fn entire_render_result() -> RenderResult {
    Ok(RenderResultStruct::entire())
}

impl Clone for RenderResultStruct {
    fn clone(&self) -> Self {
        Self {
            drawn_region: self.drawn_region.clone(),
            filter_mouse_actions: self.filter_mouse_actions,
        }
    }
}
