mod root;
mod subscriptions;
mod mouse_store;

pub use root::*;
pub use subscriptions::*;
pub use mouse_store::*;

use crate::*;

/// Every `Component` will be assigned a *buddy*. This buddy will be passed as
/// parameter to every method of the `Component` trait. The buddy is the primary
/// way the component can interact with its parent menu, or the root of knukki
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
/// information about the state of the keyboard and the mouse(s).
///
/// There are also methods that are neither subscription or read methods, the
/// remaining methods. These methods have all kinds of purposes. The most important
/// one is `request_render`, which is needed if the component wants to render itself
/// again (because of some state change for instance).
pub trait ComponentBuddy {
    /// Requests to change the parent menu component (possibly the *root*
    /// component) to the component returned by the *create_new_menu*
    /// function.
    ///
    /// ### Current menu
    /// That function should, given the *current* parent menu, create the
    /// component (menu) to replace it with. The current menu can be very
    /// useful to implement 'Back' buttons or modals (so it knows which
    /// component should be drawn in the background).
    ///
    /// ### Request
    /// Like the docs above suggest, it is a *request*: it might not happen in
    /// some rare cases (when multiple components request to change the menu at
    /// the same time, only one of them can be chosen). The buddy might reject
    /// the request for other reasons as well, but this should be uncommon.
    fn change_menu(
        &mut self,
        create_new_menu: Box<dyn Fn(Box<dyn Component>) -> Box<dyn Component>>,
    );

    /// Prompts the user to type some text for the component.
    ///
    /// This method will work even if there is no keyboard, but it will always
    /// block the entire application until the user finishes typing.
    ///
    /// The user will be asked to modify the `start_text`. The user will be
    /// able to either change the start_text and return `Some` replacement
    /// text, or cancel and return `None`.
    fn request_text_input(&self, start_text: String) -> Option<String>;

    /// Requests to re-render this component (by calling its render method)
    /// during the next frame.
    ///
    /// This request should normally not be rejected, but it could happen in
    /// rare cases (for instance when this component is detached before the
    /// next frame).
    ///
    /// Note that this component might be re-rendered even if this method is
    /// not called, for instance when the window is resized.
    fn request_render(&mut self);

    // Subscribe methods

    /// Subscribes the component for the `MouseClickEvent`
    fn subscribe_mouse_click(&mut self);

    /// Cancels the components subscription for the `MouseClickEvent`
    fn unsubscribe_mouse_click(&mut self);

    /// Subscribes the component for the `MouseClickOutEvent`
    fn subscribe_mouse_click_out(&mut self);

    /// Cancels the components subscription for the `MouseClickOutEvent`
    fn unsubscribe_mouse_click_out(&mut self);

    /// Subscribes the component for the `MouseMoveEvent`
    fn subscribe_mouse_move(&mut self);

    /// Cancels the components subscription for the `MouseMoveEvent`
    fn unsubscribe_mouse_move(&mut self);

    /// Subscribes the component for the `MouseEnterEvent`
    fn subscribe_mouse_enter(&mut self);

    /// Cancels the components subscription for the `MouseEnterEvent`
    fn unsubscribe_mouse_enter(&mut self);

    /// Subscribes the component for the `MouseLeaveEvent`
    fn subscribe_mouse_leave(&mut self);

    /// Cancels the components subscription for the `MouseLeaveEvent`
    fn unsubscribe_mouse_leave(&mut self);

    /// Subscribes the component for the `CharTypeEvent`. This method will return
    /// `Ok` if a keyboard is available, and `Err` if not. If this method returns
    /// `Err`, but the component really needs text input, it should call
    /// `request_text_input`.
    fn subscribe_char_type(&self) -> Result<(), ()>;

    /// Cancels the subscription of the component for the `CharTypeEvent`.
    fn unsubscribe_char_type(&self);

    // Read methods

    /// Gets the position of the given `Mouse` relative to the component.
    ///
    /// If the mouse cursor is currently hovering over the component, it will
    /// return Some with the relative mouse position. See the documentation of
    /// `Point` for more information about the relative coordinates.
    ///
    /// If the mouse cursor is currently not hovering over the component, this
    /// method will return None.
    ///
    /// # When this method is called while handling a mouse event
    /// If this method is called during the `on_mouse_enter` method for *mouse*, this will return
    /// the *entrance* position of that event.
    ///
    /// If this method is called during the `on_mouse_move` method for *mouse*, this will return
    /// the *new* mouse position.
    ///
    /// If this method is called during the `on_mouse_leave` method for *mouse*, this will return
    /// *None*.
    fn get_mouse_position(&self, mouse: Mouse) -> Option<Point>;

    /// Checks if the given button of the given mouse is currently being
    /// pressed/down. This method can be called during any event.
    ///
    /// If you want to check whether the *primary* button of the given mouse is
    /// pressed, the `is_primary_mouse_down` should be more convenient.
    fn is_mouse_button_down(&self, mouse: Mouse, button: MouseButton) -> bool;

    /// Checks if the primary button of the given mouse is currently being
    /// pressed/down. This method can be called during any event.
    fn is_primary_mouse_button_down(&self, mouse: Mouse) -> bool;

    /// Gets all `Mouse`s that are currently hovering over the (domain of) this component.
    ///
    /// # Filter mouse actions
    /// This method ignores the last `RenderResult`, so this will simply return all `Mouse`s that
    /// are inside the domain of the component, regardless of whether `filter_mouse_actions` was
    /// set to true.
    ///
    /// # When this method is called while handling a mouse event
    /// If this method is called during the `on_mouse_enter` for some mouse *M*, the result of
    /// this method will contain *M*.
    ///
    /// If this method is called during the `on_mouse_leave` for some mouse *M*, the result of
    /// this method *won't* contain *M*.
    fn get_local_mouses(&self) -> Vec<Mouse>;

    /// Gets all `Mouse`s that are hovering somewhere over the application window.
    ///
    /// # When this method is called while handling a mouse event
    /// If this method is called during the `fire_mouse_enter_event` of the `Application` for some
    /// mouse *M*, the result of this method will contain *M*.
    ///
    /// If this method is called during the `fire_mouse_leave_event` of the `Application` for some
    /// mouse *M*, the result of this method *won't* contain *M*.
    fn get_all_mouses(&self) -> Vec<Mouse>;
}
