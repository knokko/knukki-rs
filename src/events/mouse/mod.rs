mod button;
mod point;

pub use button::*;
pub use point::*;

/// Represents a mouse, or something else that can generate events *at screen
/// positions* (like clicking, moving, dragging...).
///
/// On phones, mouses are usually fingers. On desktops, mouses are usually the
/// mouses, but could also be controllers for instance.
///
/// ### Obtaining instances
/// All mouse events have a `get_mouse()` method to get the `Mouse` that generated
/// it. Alternatively, components can use the `get_local_mouses()` or the
/// `get_all_mouses()` method of its buddy.
///
/// ### Creating instances
/// The `new` function can be used to construct `Mouse`s, but only the
/// *provider* should do this.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Mouse {
    id: u16,
}

impl Mouse {
    /// Constructs a new `Mouse` with the given `id`. Only the *provider* should
    /// use this function.
    pub fn new(id: u16) -> Self {
        Self { id }
    }

    /// Gets the numerical id of this `Mouse`. This method is mostly useful for the
    /// *provider*, but components might also find this method useful.
    pub fn get_id(&self) -> u16 {
        self.id
    }
}

/// This event is for the `on_mouse_click` method of `Component`.
/// This event indicates that the user clicked *on* the component.
///
/// Use `MouseClickOutEvent` and the corresponding `on_mouse_click_out` method
/// to keep track of mouse clicks outside the component.
#[derive(Copy, Clone, Debug)]
pub struct MouseClickEvent {
    mouse: Mouse,
    point: MousePoint,
    button: MouseButton,
}

impl MouseClickEvent {
    /// Constructs a new `MouseClickEvent` with the given mouse, relative mouse
    /// cursor position (point) and the given button
    pub fn new(mouse: Mouse, point: MousePoint, button: MouseButton) -> Self {
        Self {
            mouse,
            point,
            button,
        }
    }

    /// Gets the `Mouse` that was clicked
    pub fn get_mouse(&self) -> Mouse {
        self.mouse
    }

    /// Gets the position of the mouse cursor, relative to the component that
    /// listens to this event
    pub fn get_point(&self) -> MousePoint {
        self.point
    }

    /// Gets the mouse button that was clicked
    pub fn get_button(&self) -> MouseButton {
        self.button
    }
}

/// This event is for the `on_mouse_click_out` method of `Component`.
/// This event indicates that the user clicked somewhere, but not on
/// the component.
///
/// Use `MouseClickEvent` and the corresponding `on_mouse_click` method to
/// keep track of mouse clicks *on* the component.
///
/// Unlike `MouseClickEvent`, this event doesn't know the mouse position,
/// but only which mouse button was used.
#[derive(Copy, Clone, Debug)]
pub struct MouseClickOutEvent {
    mouse: Mouse,
    button: MouseButton,
}

impl MouseClickOutEvent {
    /// Constructs a new `MouseClickOutEvent` with the given `Mouse` and
    /// `MouseButton`
    pub fn new(mouse: Mouse, button: MouseButton) -> Self {
        Self { mouse, button }
    }

    /// Gets the `Mouse` that was clicked
    pub fn get_mouse(&self) -> Mouse {
        self.mouse
    }

    /// Gets the `MouseButton` that was clicked
    pub fn get_button(&self) -> MouseButton {
        self.button
    }
}

/// This method is for the `on_mouse_move` method of `Component`. It indicates
/// that the user moved the mouse *within* the component: both the position
/// the mouse came from and the position the mouse went to are in the component.
///
/// ### Mouse leaving behavior
/// If the user moved the mouse from a position *f* inside the component to a
/// position *t* outside the component, a `MouseMoveEvent` will be fired from *f*
/// to the border *b* of the component where the mouse left the component.
/// Additionally, a `MouseLeaveEvent` with position *b* will be fired.
///
/// ### Mouse entering behavior
/// If the user moved the mouse from a position *f* outside the component to a
/// position *t* inside the component, a `MouseMoveEvent` will be fired from the
/// border *b* of the component where the mouse came in to *t*. Additionally,
/// a `MouseEnterEvent` with position *b* will be fired.
pub struct MouseMoveEvent {
    mouse: Mouse,
    from: MousePoint,
    to: MousePoint,
}

impl MouseMoveEvent {
    /// Constructs a new `MouseMoveEvent` indicating that `mouse` moved from
    /// `from` to `to`
    pub fn new(mouse: Mouse, from: MousePoint, to: MousePoint) -> Self {
        Self { mouse, from, to }
    }

    /// Gets the `Mouse` that was moved
    pub fn get_mouse(&self) -> Mouse {
        self.mouse
    }

    /// Gets the position the mouse cursor came from (the old mouse position)
    pub fn get_from(&self) -> MousePoint {
        self.from
    }

    /// Gets the position the mouse cursor was moved to (the new mouse position)
    pub fn get_to(&self) -> MousePoint {
        self.to
    }
}

/// The event for the `on_mouse_enter` method of `Component`. It indicates that the
/// user just moved the mouse inside the component.
///
/// This event captures the entrance_point, the position where the mouse 'set foot'
/// inside the component. For regular mouses, this position will always be on one
/// of the borders of the component. But for other 'mouses' (like fingers on phones),
/// this can be anywhere in the component.
pub struct MouseEnterEvent {
    mouse: Mouse,
    entrance_point: MousePoint,
}

impl MouseEnterEvent {
    /// Constructs a new `MouseEnterEvent` with the given `Mouse` and `entrance_point`
    pub fn new(mouse: Mouse, entrance_point: MousePoint) -> Self {
        Self {
            mouse,
            entrance_point,
        }
    }

    /// Gets the `Mouse` that entered the component
    pub fn get_mouse(&self) -> Mouse {
        self.mouse
    }

    /// Gets the position where the mouse 'set foot' inside the component.
    ///
    /// For regular mouses, this will always be on the border of the component, but
    /// this doesn't have to be the case for other `Mouse`s like fingers on
    /// touchscreens.
    pub fn get_entrance_point(&self) -> MousePoint {
        self.entrance_point
    }
}

/// The event for the `on_mouse_leave` method of `Component`. This event indicates
/// that the user just moved the mouse cursor outside the component.
///
/// This event captures the exit_point, the position where the mouse cursor
/// 'Stepped out' of the component.
///
/// For regular mouses, this will always be on the border of the component, but
/// this doesn't have to be the case for other `Mouse`s like fingers on
/// touchscreens.
pub struct MouseLeaveEvent {
    mouse: Mouse,
    exit_point: MousePoint,
}

impl MouseLeaveEvent {
    /// Constructs a new `MouseLeaveEvent` with the given `exit_point`
    pub fn new(mouse: Mouse, exit_point: MousePoint) -> Self {
        Self { mouse, exit_point }
    }

    /// Gets the `Mouse` that left the component
    pub fn get_mouse(&self) -> Mouse {
        self.mouse
    }

    /// Gets the position where the mouse left the component.
    ///
    /// For regular mouses, this will always be on the border of the component, but
    /// this doesn't have to be the case for other `Mouse`s like fingers on
    /// touchscreens.
    pub fn get_exit_point(&self) -> MousePoint {
        self.exit_point
    }
}
