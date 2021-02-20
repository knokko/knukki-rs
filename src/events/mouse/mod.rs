mod button;

use crate::Point;

pub use button::*;

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
    point: Point,
    button: MouseButton,
}

impl MouseClickEvent {
    /// Constructs a new `MouseClickEvent` with the given mouse, relative mouse
    /// cursor position (point) and the given button
    pub fn new(mouse: Mouse, point: Point, button: MouseButton) -> Self {
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
    pub fn get_point(&self) -> Point {
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

/// This event is for the `on_mouse_press` method of `Component`. It indicates that the user has
/// pressed a mouse button **on** the component.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct MousePressEvent {
    mouse: Mouse,
    point: Point,
    button: MouseButton,
}

impl MousePressEvent {
    /// Constructs a new `MousePressEvent` with the given `Mouse`, `Point`, and `MouseButton`.
    pub fn new(mouse: Mouse, point: Point, button: MouseButton) -> Self {
        Self {
            mouse,
            point,
            button,
        }
    }

    /// Gets the `Mouse` that was pressed.
    pub fn get_mouse(&self) -> Mouse {
        self.mouse
    }

    /// Gets the `Point` where the mouse was pressed.
    pub fn get_point(&self) -> Point {
        self.point
    }

    /// Gets the `MouseButton` that was pressed.
    pub fn get_button(&self) -> MouseButton {
        self.button
    }
}

/// This event is for the `on_mouse_release` method of `Component`. It indicates that the user has
/// released a mouse button **on** the component.
///
/// Note: when the user releases the mouse quickly after pressing it, a `MouseClickEvent` will be
/// fired after this event is fired.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct MouseReleaseEvent {
    mouse: Mouse,
    point: Point,
    button: MouseButton,
}

impl MouseReleaseEvent {
    /// Constructs a new `MouseReleaseEvent` with the given `Mouse`, `Point`, and `MouseButton`.
    pub fn new(mouse: Mouse, point: Point, button: MouseButton) -> Self {
        Self {
            mouse,
            point,
            button,
        }
    }

    /// Gets the `Mouse` that was released.
    pub fn get_mouse(&self) -> Mouse {
        self.mouse
    }

    /// Gets the `Point` where the mouse was released.
    pub fn get_point(&self) -> Point {
        self.point
    }

    /// Gets the `MouseButton` that was released.
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
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct MouseMoveEvent {
    mouse: Mouse,
    from: Point,
    to: Point,
}

impl MouseMoveEvent {
    /// Constructs a new `MouseMoveEvent` indicating that `mouse` moved from
    /// `from` to `to`
    pub fn new(mouse: Mouse, from: Point, to: Point) -> Self {
        Self { mouse, from, to }
    }

    /// Gets the `Mouse` that was moved
    pub fn get_mouse(&self) -> Mouse {
        self.mouse
    }

    /// Gets the position the mouse cursor came from (the old mouse position)
    pub fn get_from(&self) -> Point {
        self.from
    }

    /// Gets the position the mouse cursor was moved to (the new mouse position)
    pub fn get_to(&self) -> Point {
        self.to
    }

    /// Gets the distance the mouse travelled in the x-direction. This method simply returns
    /// `to.get_x() - from.get_x()`.
    pub fn get_delta_x(&self) -> f32 {
        self.to.get_x() - self.from.get_x()
    }

    /// Gets the distance the mouse travelled in the y-direction. This method simply returns
    /// `to.get_y() - from.get_y()`.
    pub fn get_delta_y(&self) -> f32 {
        self.to.get_y() - self.from.get_y()
    }
}

/// The event for the `on_mouse_enter` method of `Component`. It indicates that the
/// user just moved the mouse inside the component.
///
/// This event captures the entrance_point, the position where the mouse 'set foot'
/// inside the component. For regular mouses, this position will always be on one
/// of the borders of the component. But for other 'mouses' (like fingers on phones),
/// this can be anywhere in the component.
///
/// If this event comes directly from the provider and any mouse buttons are pressed, the provider
/// will fire `MousePressEvent`s right after this event. This is needed by the `Application` to
/// update the pressed buttons in its `MouseStore`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MouseEnterEvent {
    mouse: Mouse,
    entrance_point: Point,
}

impl MouseEnterEvent {
    /// Constructs a new `MouseEnterEvent` with the given `Mouse` and `entrance_point`
    pub fn new(mouse: Mouse, entrance_point: Point) -> Self {
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
    pub fn get_entrance_point(&self) -> Point {
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
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MouseLeaveEvent {
    mouse: Mouse,
    exit_point: Point,
}

impl MouseLeaveEvent {
    /// Constructs a new `MouseLeaveEvent` with the given `exit_point`
    pub fn new(mouse: Mouse, exit_point: Point) -> Self {
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
    pub fn get_exit_point(&self) -> Point {
        self.exit_point
    }
}
