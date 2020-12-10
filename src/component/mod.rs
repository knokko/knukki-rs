use crate::*;

mod area;
mod buddy;

pub use area::*;
pub use buddy::*;

/// The core trait of this crate. `Component`s are basically event handlers for
/// gui events like mouse events and keyboard events, but most importantly render
/// events to draw themselves on a Golem context.
/// 
/// There are simple components like buttons and checkboxes, but also complicated
/// menu components that are composed of multiple other 'child' components. 
/// Such menu components would propagate the events it receives to its child
/// components.
/// 
/// In running knukki applications, there will be a single *root* component
/// that will be drawn over the entire window or screen. This is the only
/// component that will receive events directly: all other components can
/// only receive the events that are propagated to them by the *root*.
pub trait Component {

    fn on_attach(&mut self, _buddy: &mut dyn ComponentBuddy) {}

    fn on_mouse_click(&mut self, _event: MouseClickEvent, _buddy: &mut dyn ComponentBuddy) {}

    fn on_mouse_click_out(&mut self, _event: MouseClickOutEvent) {}

    fn on_mouse_move(&mut self, _event: MouseMoveEvent) {}

    fn on_mouse_enter(&mut self, _event: MouseEnterEvent) {}

    fn on_mouse_leave(&mut self, _event: MouseLeaveEvent) {}

    fn on_char_type(&mut self, _event: &CharTypeEvent) {}

    fn on_detach(&mut self, _buddy: &mut dyn ComponentBuddy) {}
}