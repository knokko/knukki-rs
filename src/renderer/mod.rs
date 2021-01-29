use crate::RenderRegion;
use std::cell::RefCell;

mod core;
#[cfg(feature = "golem_rendering")]
mod golem_renderer;
#[cfg(feature = "golem_rendering")]
pub use golem_renderer::ShaderId;

pub struct Renderer {
    #[cfg(feature = "golem_rendering")]
    context: golem::Context,
    #[cfg(feature = "golem_rendering")]
    storage: golem_renderer::GolemRenderStorage,
    viewport_stack: RefCell<Vec<RenderRegion>>,
    scissor_stack: RefCell<Vec<RenderRegion>>,
}

#[cfg(test)]
#[cfg(not(feature = "golem_rendering"))]
pub(crate) fn test_renderer(initial_viewport: RenderRegion) -> Renderer {
    Renderer {
        viewport_stack: RefCell::new(vec![initial_viewport]),
        scissor_stack: RefCell::new(vec![initial_viewport]),
    }
}
