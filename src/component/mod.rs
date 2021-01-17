use crate::*;

mod buddy;
mod dummy;
mod render;

pub use buddy::*;
pub use dummy::*;
pub use render::*;

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
    fn on_attach(&mut self, buddy: &mut dyn ComponentBuddy);

    fn on_resize(&mut self, _buddy: &mut dyn ComponentBuddy) {}

    /// Lets this component render itself, and returns some information about the rendering.
    ///
    /// # The rendering
    /// Whenever this method is called, the component should render itself using
    /// the given golem `Context`. The given `region` is just for the information
    /// of the component: it can ignore it because the caller must ensure that the
    /// viewport is set accordingly.
    ///
    /// # Fake rendering
    /// When the `golem_rendering` feature is not enabled, there will not be
    /// a *golem* parameter for the actual drawing. The component is then
    /// supposed to return the same `RenderResult` as if there were a real
    /// golem context, but without the actual drawing. Doing this accurately
    /// makes unit testing easier.
    ///
    /// # When this method will be called
    /// This method will only be called if the component asked for it via the
    /// `request_render` method of its buddy, or the parent (or provider)
    /// determined that it was necessary (for instance, the window was resized
    /// or the operating system requested it). In the latter case, the value
    /// of the parameter *force* will be true. If *force* is true, the
    /// component is supposed to redraw itself completely. If *force* is false,
    /// the component should only redraw the things that changed since the
    /// previous call to `render`.
    ///
    /// # Continuous rendering
    /// If you want this method to be called continuously, you should call the
    /// `request_render` method of `buddy` during this method call (which
    /// basically requests to be rendered again as soon as possible). This
    /// method will also be called soon after the component is attached.
    ///
    /// # The return value
    /// ## The drawn area
    /// For the sake of optimization, it is very interesting for the caller to
    /// know where the component did and did not render stuff (so that it knows
    /// which other components need to be redrawn). The `drawn_area` field of
    /// the return value is used for this. For some component, this is always
    /// the entire *region*, but other components may wish to ignore a part
    /// of their *region* to avoid distortion or because they want to draw
    /// something that doesn't fill an entire rectangle (for instance a circle).
    ///
    /// ## Filter mouse actions
    /// Furthermore, it is possible (and optional) to set the `filter_mouse_actions`
    /// field of the return value. If that is true and this component is
    /// subscribed to mouse events, only the region(s) where the component has
    /// drawn something will be considered as the component domain. As example,
    /// consider the situation where the component is registered for the
    /// `MouseClickEvent` and the user clicks inside the component domain, but
    /// *outside* the `drawn_area`. If `filter_mouse_actions` is set to false,
    /// the `on_mouse_click` method of the component will be called because
    /// the click happened inside its domain. But if `filter_mouse_actions` is
    /// set to true, it will *not* be called because the component didn't render
    /// there. This can be convenient for many clickable components that don't
    /// always use their full component domain.
    ///
    /// ### Affected mouse events
    /// The following mouse events will be affected by `filter_mouse_actions`: `MouseClickEvent`,
    /// `MouseClickOutEvent`, `MouseMoveEvent`, `MouseEnterEvent`, and `MouseLeaveEvent`.
    ///
    /// This will *not* affect the `get_local_mouses` method of this buddy.
    fn render(
        &mut self,
        renderer: &Renderer,
        buddy: &mut dyn ComponentBuddy,
        force: bool,
    ) -> RenderResult;

    fn on_mouse_click(&mut self, _event: MouseClickEvent, _buddy: &mut dyn ComponentBuddy) {
        forgot("MouseClick")
    }

    fn on_mouse_click_out(&mut self, _event: MouseClickOutEvent, _buddy: &mut dyn ComponentBuddy) {
        forgot("MouseClickOut")
    }

    fn on_mouse_move(&mut self, _event: MouseMoveEvent, _buddy: &mut dyn ComponentBuddy) {
        forgot("MouseMove")
    }

    fn on_mouse_enter(&mut self, _event: MouseEnterEvent, _buddy: &mut dyn ComponentBuddy) {
        forgot("MouseEnter")
    }

    fn on_mouse_leave(&mut self, _event: MouseLeaveEvent, _buddy: &mut dyn ComponentBuddy) {
        forgot("MouseLeave")
    }

    fn on_char_type(&mut self, _event: &CharTypeEvent) {
        forgot("CharType")
    }

    fn on_detach(&mut self) {
        // Components don't register for this event explicitly and many events
        // won't need to implement this, so no need for a panic
    }
}

fn forgot(event_name: &'static str) -> ! {
    panic!(
        "This component registered itself for the {}Event, 
    but didn't implement the event handler for it",
        event_name
    )
}
