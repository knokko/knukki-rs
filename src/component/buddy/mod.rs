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
/// information about the state of the keyboard and the mouse.
/// 
/// There are also methods that are neither subscription or read methods, the
/// remaining methods. These methods have all kinds of purposes. The import ones
/// are `request_render` and `set_used_area`. `request_render` is needed to make
/// sure the component will be re-rendered the next frame and `set_used_area`
/// should be used to tell the parent what part of the component domain is
/// actually being used. 
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
    fn change_menu(&self, create_new_menu: impl Fn(Box<dyn Component>) -> Box<dyn Component>);

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
    fn request_render(&self);

    /// Notifies this buddy that the used area of the component has been changed.
    /// 
    /// To elaborate a bit: every `Component` will get a domain (a part of the
    /// window/screen) in which it can render and receive events. For the root
    /// component, that will be the entire window/screen. 
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
    /// `MousePoint` for more information about the relative coordinates.
    /// 
    /// If the mouse cursor is currently not hovering over the component, this 
    /// method will return None.
    fn get_mouse_position(&self, mouse: Mouse) -> Option<MousePoint>;

    /// Checks if the given button of the given mouse is currently being 
    /// pressed/down. This method can be called during any event. 
    /// 
    /// If you want to check whether the *primary* button of the given mouse is
    /// pressed, the `is_primary_mouse_down` should be more convenient.
    fn is_mouse_down(&self, mouse: Mouse, button: MouseButton) -> bool;

    /// Checks if the primary button of the given mouse is currently being
    /// pressed/down. This method can be called during any event.
    fn is_primary_mouse_down(&self, mouse: Mouse) -> bool;

    /// Gets all `Mouse`s that are currently hovering over this component
    fn get_local_mouses(&self) -> Vec<Mouse>;

    /// Gets all `Mouse`s that are hovering somewhere over the application window.
    fn get_all_mouses(&self) -> Vec<Mouse>;

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