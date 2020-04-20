/// Represents a mouse position relative to the position of a *Component*.
/// *MousePoint*s will typically be used as properties of mouse events, and
/// they will be relative to the component who's on_mouse_xxx() methods is
/// being called.
/// 
/// The point (0.0, 0.0) represents the bottom-left corner of the component
/// and the point (1.0, 1.0) represents the top-right corner of the component.
#[derive(Clone,Copy,Debug)]
pub struct MousePoint {

    x: f32,
    y: f32
}

impl MousePoint {

    /// Constructs a new *MousePoint* with the given *x* and *y*
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Gets the (relative) x coordinate of this *MousePoint*. 
    /// A value of 0.0 indicates that this mouse point is on the left bound
    /// of the component and a value of 1.0 indicates that this mouse point
    /// is on the right bound of the component.
    pub fn get_x(&self) -> f32 {
        self.x
    }

    /// Gets the (relative) y coordinate of this *MousePoint*.
    /// A value of 0.0 indicates that this mouse point is on the bottom bound
    /// of the component and a value of 1.0 indicates that this mouse point is
    /// on the top bound of the component.
    pub fn get_y(&self) -> f32 {
        self.y
    }
}