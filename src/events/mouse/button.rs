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
    pub fn is_left(&self) -> bool { *self == LEFT_MOUSE_BUTTON }

    /// Returns true if this button is the right mouse button
    pub fn is_right(&self) -> bool { *self == RIGHT_MOUSE_BUTTON }

    /// Returns true if this button is the mouse wheel (most mouse wheels can
    /// not only be used for scrolling, but can also be pressed and released,
    /// which allow it to be used as a button).
    pub fn is_wheel(&self) -> bool { *self == MOUSE_WHEEL_BUTTON }
}

/// A constant representating the left mouse button. This constant should be
/// convenient when calling methods that check if a certain mouse button is
/// down. 
pub const LEFT_MOUSE_BUTTON: MouseButton = MouseButton::new(0);

/// A constant representating the right mouse button. This constant should be
/// convenient when calling methods that check if a certain mouse button is
/// down. 
pub const RIGHT_MOUSE_BUTTON: MouseButton = MouseButton::new(2);

/// A constant representating the 'mouse wheel button`. The mouse wheel is
/// normally used to scroll, but most mouse wheels can also be pressed. 
/// This constant should be convenient when calling methods that check if a 
/// certain mouse button is down. 
pub const MOUSE_WHEEL_BUTTON: MouseButton = MouseButton::new(1);

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_left() {

        // Positive tests
        assert!(LEFT_MOUSE_BUTTON.is_left());
        assert!(MouseButton::new(0).is_left());
        assert!(MouseButton::new(0) == LEFT_MOUSE_BUTTON);

        // Negative tests
        assert!(!RIGHT_MOUSE_BUTTON.is_left());
        assert!(!MouseButton::new(4).is_left());
        assert!(MouseButton::new(4) != LEFT_MOUSE_BUTTON);
    }

    #[test]
    fn test_right() {

        // Positive tests
        assert!(RIGHT_MOUSE_BUTTON.is_right());
        assert!(MouseButton::new(2).is_right());
        assert!(MouseButton::new(2) == RIGHT_MOUSE_BUTTON);

        // Negative tests
        assert!(!LEFT_MOUSE_BUTTON.is_right());
        assert!(!MouseButton::new(4).is_right());
        assert!(MouseButton::new(4) != RIGHT_MOUSE_BUTTON);
    }

    #[test]
    fn test_wheel() {
        assert!(MOUSE_WHEEL_BUTTON.is_wheel());
        assert!(MouseButton::new(1).is_wheel());
        assert!(MouseButton::new(1) == MOUSE_WHEEL_BUTTON);

        // Negative tests
        assert!(!LEFT_MOUSE_BUTTON.is_wheel());
        assert!(!MouseButton::new(4).is_wheel());
        assert!(MouseButton::new(4) != MOUSE_WHEEL_BUTTON);
    }

    #[test]
    fn test_other() {

        assert!(MouseButton::new(5).get_index() == 5);
        assert!(MouseButton::new(6) == MouseButton::new(6));
        assert!(MouseButton::new(5) != MouseButton::new(6));
    }
}