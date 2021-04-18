use crate::RenderRegion;
use std::cell::RefCell;

mod core;
#[cfg(feature = "golem_rendering")]
mod golem_renderer;

mod text;

#[cfg(feature = "golem_rendering")]
pub use golem_renderer::ShaderId;

pub use text::*;

/// This struct is used to render `Component`s (and the `Application`). A reference to an instance
/// of this struct will be passed as parameter to every `render` method.
///
/// ## Methods
/// This struct has *core* methods and *feature* methods. The *core* methods will always be
/// available, regardless of compile target and whether or not there is an actual render target.
/// The *feature* methods are only available when the right crate feature is enabled. Currently,
/// the `golem_rendering` feature is the only crate feature that adds *feature* methods.
///
/// ## Usage
/// `Component`s should use `#[cfg(feature = "golem_rendering")]` before code blocks that need to
/// use *feature* methods. It is encouraged to always use the same *core* methods, regardless of
/// which features are enabled: even though no real drawing will be done without crate features, it
/// is still nice for unit testing. An example usage is shown below:
/// ```
/// use knukki::*;
///
/// fn render_stuff(renderer: &Renderer) {
///     // Use the core push_viewport method
///     renderer.push_viewport(0.2, 0.2, 0.8, 0.8, || {
///         // Use the core clear method
///         renderer.clear(Color::rgb(100, 100, 0));
///
///         #[cfg(feature = "golem_rendering")]
///         {
///             let context = renderer.get_context();
///             // Do some more complicated rendering using the golem context
///             // Or use some other feature methods
///         }
///     });
/// }
/// ```
/// ## Constructing instances
/// The *wrapper* is responsible for constructing the `Renderer`(s). In production environments, it
/// will construct a real `Renderer` from a `golem` `Context`. Unit tests can use the
/// `test_renderer` function to easily construct a dummy `Renderer`.
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
