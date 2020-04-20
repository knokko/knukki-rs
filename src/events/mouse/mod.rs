mod button;
mod point;

pub use button::*;
pub use point::*;

use button::MouseButton;
use point::MousePoint;

/// The event is for the on_mouse_click method of *Component*.
/// This event indicates that the user clicked *on* the component.
/// 
/// Use *MouseClickOutEvent* and the corresponding on_mouse_click_out method
/// to keep track of mouse clicks outside the component.
pub struct MouseClickEvent {

    point: MousePoint,
    button: MouseButton
}

impl MouseClickEvent {

    /// Constructs a new *MouseClickEvent* with the given relative mouse
    /// cursor position (point) and the given button
    pub fn new(point: MousePoint, button: MouseButton) -> Self {
        Self { point, button }
    }

    /// Gets the position of the mouse cursor, relative to the component that
    /// listens to this event
    pub fn get_point(&self) -> MousePoint {
        self.point
    }

    /// Gets the mouse button that was clicked with
    pub fn get_button(&self) -> MouseButton {
        self.button
    }
}

/// This event is for the on_mouse_click_out method of *Component*. 
/// This event indicates that the user clicked somewhere, but not on
/// the component. 
/// 
/// Use *MouseClickEvent* and the corresponding on_mouse_click method to
/// keep track of mouse clicks *on* the component.
/// 
/// Unlike MouseClickEvent, this event doesn't know the mouse position,
/// but only which mouse button was used.
pub struct MouseClickOutEvent {
    
    button: MouseButton
}

impl MouseClickOutEvent {

    pub fn new(button: MouseButton) -> Self { Self { button } }

    /// Gets the mouse button that was clicked with
    pub fn get_button(&self) -> MouseButton { self.button }
}

/// This method is for the on_mouse_move method of *Component*. It indicates
/// that the user moved the mouse *within* the component: both the position
/// the mouse came from and the position the mouse went to are in the component.
/// 
/// If the user moved the mouse from a position *f* inside the component to a 
/// position *t* outside the component, a MouseMoveEvent will be fired from *f*
/// to the border *b* of the component where the mouse left the component. 
/// Additionally, a MouseLeaveEvent with position *b* will be fired.
/// 
/// If the user moved the mouse from a position *f* outside the component to a
/// position *t* inside the component, a MouseMoveEvent will be fired from the
/// border *b* of the component where the mouse came in to *t*. Additionally,
/// a MouseEnterEvent with position *b* will be fired.
pub struct MouseMoveEvent {

    from: MousePoint,
    to: MousePoint
}

impl MouseMoveEvent {

    pub fn new(from: MousePoint, to: MousePoint) -> Self { Self { from, to } }

    /// Gets the position the mouse cursor came from (the old mouse position)
    pub fn get_from(&self) -> MousePoint { self.from }

    /// Gets the position the mouse cursor was moved to (the new mouse position)
    pub fn get_to(&self) -> MousePoint { self.to }
}

/// The event for the on_mouse_enter method of *Component*. It indicates that the
/// user just moved the mouse inside the component.
/// 
/// This event captures the entrance_point, a position at the border of the 
/// component at which the mouse entered the component. 
pub struct MouseEnterEvent {

    entrance_point: MousePoint
}

impl MouseEnterEvent {

    pub fn new(entrance_point: MousePoint) -> Self { Self { entrance_point } }

    /// Gets the position where the mouse entered the component. This position
    /// will always be at one of the borders of the component (or very close due
    /// to rounding errors of floating point numbers).
    pub fn get_entrance_point(&self) -> MousePoint { self.entrance_point }
}

/// The event for the on_mouse_leave method of *Component*. This event indicates
/// that the user just moved the mouse cursor outside the component. 
/// 
/// This event captures the exit_point, a position at the border of the component
/// where the mouse left the component.
pub struct MouseLeaveEvent {

    exit_point: MousePoint
}

impl MouseLeaveEvent {

    pub fn new(exit_point: MousePoint) -> Self { Self { exit_point } }

    /// Gets the position where the mouse left the component. This position will
    /// always be at the border of the component (or almost due to rounding errors
    /// of floating point numbers).
    pub fn get_exit_point(&self) -> MousePoint { self.exit_point }
}