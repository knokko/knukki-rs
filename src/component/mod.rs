use crate::*;

/// The core trait of this crate. `Component`s are basically event handlers for
/// gui events like mouse events and keyboard events, but most importantly render
/// events to draw themselves on a WebGl context.
/// 
/// There are simple components like buttons and checkboxes, but also complicated
/// menu components that are composed of multiple other 'child' components. 
/// Such menu components would propagate the events it receives to its child
/// components.
/// 
/// In running wasmuri applications, there will be a big html canvas spanning
/// the full browser tab and there will be a single root component that will draw
/// on this canvas and whose event handling methods will be called each time an 
/// event is received from the browser. This root component will typically be a 
/// menu component that propagates the events to its child components.
pub trait Component {

    fn on_attach(&mut self);

    fn on_mouse_click(&mut self, event: MouseClickEvent);

    fn on_mouse_click_out(&mut self, event: MouseClickOutEvent);

    fn on_mouse_move(&mut self, event: MouseMoveEvent);

    fn on_mouse_enter(&mut self, event: MouseEnterEvent);

    fn on_mouse_leave(&mut self, event: MouseLeaveEvent);

    fn on_detach(&mut self);
}