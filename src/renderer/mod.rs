#[cfg(feature = "golem_rendering")]
mod golem_renderer;

#[cfg(not(feature = "golem_rendering"))]
pub type Renderer = ();

#[cfg(feature = "golem_rendering")]
pub use golem_renderer::*;