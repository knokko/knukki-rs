use crate::*;

/// Every `Component` will be assigned a *buddy*. This buddy will be passed as
/// parameter to every method of the `Component` trait. The buddy is the primary
/// way the component can interact with its parent menu, or the root of wasmuri
/// if there is no parent menu. `ComponentBuddy` has subscribe methods, read 
/// methods and others methods. 
/// 
/// The subscribe methods can be used to subscribe the component for certain
/// *events* (for instance `MouseClickEvent` and `KeyPressEvent`). *Until the
/// component calls the corresponding subscribe method, its event handling
/// methods will not be called.* Exceptions are made for the `on_attach` and
/// `on_detach` methods that will always be called. For each subscribe method,
/// there is also an unsubscribe method to cancel the event listen subscription.
/// 
/// The read methods can be used to query information from the parent, mostly 
/// information about the state of the keyboard and the mouse.
/// 
/// There are also methods that are neither subscription or read methods, the
/// remaining methods. These methods have all kinds of purposes. The import ones
/// are `request_render` and `set_used_area`. `request_render` is needed to make
/// sure the component will be re-rendered the next frame and `set_used_area`
/// should be used to tell the parent what part of the component domain is
/// actually being used. 
pub trait ComponentBuddy {

    /// Requests to change the parent menu component to *new_menu*. If this
    /// component doesn't have a parent menu component (for instance because
    /// it is the root component of wasmuri), a request will be made to change
    /// the root component of wasmuri to *new_menu*. 
    /// 
    /// Like the docs above suggest, it is a *request*: it might not happen in
    /// some rare cases (when multiple components request to change the menu at
    /// the same time, only one of them can be chosen). The buddy might reject
    /// the request for other reasons as well, but this should be uncommon.
    fn change_menu(&self, new_menu: Box<dyn Component>);

    // TODO Add show_modal

    /// Requests to re-render this component (by calling its render method) 
    /// during the next frame.
    /// 
    /// This request should normally not be rejected, but it could happen in
    /// rare cases (for instance when this component is detached before the
    /// next frame).
    /// 
    /// Note that this component might be re-rendered even if this method is
    /// not called, for instance when the window is resized.
    fn request_render(&self);

    /// Notifies this buddy that the used area of the component has been changed.
    /// 
    /// To elaborate a bit: every `Component` will get a domain (a part of the
    /// browser window) in which it can render and receive events. For the root
    /// component, that will be the entire browser window. 
    /// 
    /// However, components do not have to use their entire domain the whole time: 
    /// they might also only use a part of it or let it vary over time. For
    /// instance, text components will typically try to prevent their text from
    /// being 'stretched out' by not using the full horizontal or vertical range of
    /// their domain.
    /// 
    /// This method should be used to let its buddy know which part of the domain the
    /// component is currently using and should be called again whenever this part
    /// changes. Giving accurate component areas and calling this method on time will
    /// improve the accuracy of the ui.
    fn set_used_area(&self, area: Box<dyn ComponentArea>);

    // Subscribe methods

    /// Subscribes the component for the `MouseClickEvent`
    fn subscribe_mouse_click(&self);

    /// Cancels the components subscription for the `MouseClickEvent`
    fn unsubscribe_mouse_click(&self);

    /// Subscribes the component for the `MouseClickOutEvent`
    fn subscribe_mouse_click_out(&self);

    /// Cancels the components subscription for the `MouseClickOutEvent`
    fn unsubscribe_mouse_click_out(&self);

    /// Subscribes the component for the `MouseMoveEvent`
    fn subscribe_mouse_move(&self);

    /// Cancels the components subscription for the `MouseMoveEvent`
    fn unsubscribe_mouse_move(&self);

    /// Subscribes the component for the `MouseEnterEvent`
    fn subscribe_mouse_enter(&self);

    /// Cancels the components subscription for the `MouseEnterEvent`
    fn unsubscribe_mouse_enter(&self);

    /// Subscribes the component for the `MouseLeaveEvent`
    fn subscribe_mouse_leave(&self);

    /// Cancels the components subscription for the `MouseLeaveEvent`
    fn unsubscribe_mouse_leave(&self);

    // Read methods

    /// Gets the mouse position relative to the component. 
    /// 
    /// If the mouse cursor is currently hovering over the component, it will
    /// return Some with the relative mouse position. See the documentation of
    /// `MousePoint` for more information about the relative coordinates.
    /// 
    /// If the mouse cursor is currently not hovering over the component, this 
    /// method will return None.
    fn get_mouse_position(&self) -> Option<MousePoint>;

    /// Checks if the given mouse button is currently being pressed/down. This
    /// method can be called during any event. 
    /// 
    /// The constants `LEFT_MOUSE_BUTTON`, `RIGHT_MOUSE_BUTTON` and 
    /// `MOUSE_WHEEL_BUTTON` can be used as parameter for this method, but you
    /// can also create your own instances of `MouseButton` and use those.
    fn is_mouse_down(button: MouseButton) -> bool;

    /// Gets the aspect ratio of the domain of the component, that is, the width
    /// of the domain divided by the height of the domain. 
    /// 
    /// Components can use the aspect ratio to draw shapes without distortion
    /// (to make sure that the squares they draw have the same width as height on
    /// the screen). 
    /// 
    /// The size of the domain (for instance in pixels) will *not* be made 
    /// available: the aspect ratio is all information components can get.
    /// This is intentional because components should normally not need such
    /// information (but this might change in the future).
    fn get_aspect_ratio(&self) -> f32;
}