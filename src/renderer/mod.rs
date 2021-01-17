#[cfg(feature = "golem_rendering")]
mod golem_renderer;

pub struct RendererStruct {
    #[cfg(feature = "golem_rendering")] context: golem::Context
}

pub type Renderer<'a> = &'a RendererStruct;

#[cfg(test)]
#[cfg(not(feature = "golem_rendering"))]
pub(crate) fn test_renderer<'a>() -> Renderer<'a> {
    &RendererStruct {}
}