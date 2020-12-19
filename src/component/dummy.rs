use crate::*;

/// An implementation of `Component` that is not meant to be attached, but is used
/// as a work-around for swapping Components: Rust requires struct fields to have
/// a value at all times, which can be inconvenient while swapping.
pub struct DummyComponent {}

impl Component for DummyComponent {
    fn on_attach(&mut self, _buddy: &mut dyn ComponentBuddy) {
        panic!("Dummy components shouldn't be attached");
    }

    #[cfg(feature = "golem_rendering")]
    fn render(
        &mut self,
        _golem: &golem::Context,
        _region: RenderRegion,
        _buddy: &mut dyn ComponentBuddy,
    ) -> RenderResult {
        panic!("Dummy components shouldn't be asked to render itself");
    }
}
