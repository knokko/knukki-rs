/// Represents one of the buttons of a mouse, for instance the left button
/// or the right button. 
/// 
/// This struct is typically used for mouse events to indicate which button
/// was clicked or pressed.
/// 
/// Most users would simply use the `is_left`, `is_right`
/// and `is_wheel` methods to check which mouse button it was, but there is
/// also a `get_index` method to get the numerical index of the button. That
/// method can be used to track for instance macro mouse buttons.
#[derive(Clone,Copy,PartialEq,Eq,Debug)]
pub struct MouseButton {

    index: u8
}

impl MouseButton {

    pub const fn new(index: u8) -> Self { Self { index } }

    /// Gets the numerical index of this mouse button. 
    /// 
    /// Normally, a value of 0 indicates a left click, a value of 1 indicates a 
    /// mouse wheel click and a value of 2 indicates a right click. Other values
    /// indicate that some other button was used, for instance a macro button. 
    /// 
    /// If you are only interested in the common mouse buttons (left, wheel
    /// and right), you can use the `is_left`, `is_right` and `is_wheel` methods
    /// instead of this method.
    pub fn get_index(&self) -> u8 { self.index }

    /// Returns true if this button is the left mouse button
    pub fn is_left(&self) -> bool { self.index == 0 }

    /// Returns true if this button is the right mouse button
    pub fn is_right(&self) -> bool { self.index == 2 }

    /// Returns true if this button is the mouse wheel (most mouse wheels can
    /// not only be used for scrolling, but can also be pressed and released,
    /// which allow it to be used as a button).
    pub fn is_wheel(&self) -> bool { self.index == 1 }
}
