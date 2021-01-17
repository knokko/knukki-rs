use crate::RenderRegion;

#[cfg(feature = "golem_rendering")]
mod golem_renderer;
mod core;

pub struct Renderer {
    #[cfg(feature = "golem_rendering")] context: golem::Context,
    viewport_stack: Vec<RenderRegion>,
    scissor_stack: Vec<RenderRegion>,
}

#[cfg(test)]
#[cfg(not(feature = "golem_rendering"))]
pub(crate) fn test_renderer(initial_viewport: RenderRegion) -> Renderer {
    Renderer {
        viewport_stack: vec![initial_viewport],
        scissor_stack: vec![initial_viewport],
    }
}

// TODO Update the documentation of the render methods of Application and Component