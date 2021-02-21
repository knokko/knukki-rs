/// Represents one of the buttons of a `Mouse`, for instance the primary button,
///
/// This struct is typically used for mouse events to indicate which button
/// was clicked or pressed.
///
/// Every `Mouse` has a *primary* button and 0 or more additional buttons. The
/// `is_primary` method can be used to check whether a button is the primary
/// button. There is also the `get_index` method, which can be used to distinguish
/// the other buttons.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MouseButton {
    index: u8,
}

impl MouseButton {
    /// Constructs a new `MouseButton` with the given `index`. This function
    /// should normally only be used by the *wrapper*.
    pub const fn new(index: u8) -> Self {
        Self { index }
    }

    /// Constructs an instance of `MouseButton` that represents the *primary*
    /// button of a `Mouse`.
    pub const fn primary() -> Self {
        Self { index: 0 }
    }

    /// Gets the numerical index of this mouse button.
    ///
    /// This will always be 0 for the primary button, and some other value for
    /// the other buttons.
    ///
    /// This method is particularly useful to distinguish the other buttons from
    /// each other. Use the `is_primary` button if you just want to check whether
    /// this is the primary mouse button.
    ///
    /// # Index conventions
    /// To keep this crate cross-platform, there are very little rules that describe the meaning of
    /// the index of a `MouseButton`. To help applications that focus on a particular platform and
    /// need to know which button it is, here are some conventions:
    ///
    /// ## Desktop mouse
    /// - 0 (primary) is the left mouse button
    /// - 1 is the right mouse button
    /// - 2 is the mouse wheel button
    /// - 3 and higher are macro buttons
    ///
    /// ## Mobile 'mouse'
    /// - 0 (primary) is the finger
    /// - [Experimental] 1 and higher can be used to indicate some special touch devices. I will
    /// stabilize this when I do more research into mobile events (and there is a mobile wrapper
    /// available)
    ///
    /// ## Controller device/mouse
    /// I will standardize this when I do research into this and add support in the wrappers.
    pub fn get_index(&self) -> u8 {
        self.index
    }

    /// Checks whether this mouse button is the primary buttons (and returns true if
    /// and only if that is the case)
    pub fn is_primary(&self) -> bool {
        self.index == 0
    }
}

#[cfg(test)]
mod tests {
    use crate::MouseButton;

    #[test]
    fn test_primary() {
        assert!(MouseButton::primary().is_primary());
        assert_eq!(0, MouseButton::primary().get_index());
    }

    #[test]
    fn test_new() {
        assert!(!MouseButton::new(3).is_primary());
        assert_eq!(3, MouseButton::new(3).get_index());
    }
}
