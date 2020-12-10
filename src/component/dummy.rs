use super::Component;

/// An implementation of `Component` that does absolutely nothing. This struct is
/// currently used as work-around for situations where components need to be
/// swapped.
pub struct DummyComponent {}

impl Component for DummyComponent {}