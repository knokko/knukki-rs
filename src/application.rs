use crate::*;

/// The `Application` is the 'highest' object that is cross-platform. It
/// encapsulates all the components and their buddies.
/// 
/// The application has methods to fire events to the components and to
/// render them. It is the responsibility of the *provider* to make
/// sure these methods are called when appropriate.
/// 
/// The application knows nothing about the *provider*: it doesn't even
/// know whether it is being controlled by a real user or an automatic
/// testing program (except that the latter one will probably not call
/// the render method).
/// 
/// This has the interesting implication that an application can be tested
/// with regular unit tests, without needing any kind of window or
/// browser environment.
pub struct Application {

    root_component: Box<dyn Component>,
    root_buddy: RootComponentBuddy
}

impl Application {

    pub fn new(mut initial_root_component: Box<dyn Component>) -> Self {
        let mut root_buddy = RootComponentBuddy::new();
        initial_root_component.on_attach(&mut root_buddy);
        Self {
            root_component: initial_root_component,
            root_buddy
        }
    }

    pub fn fire_mouse_click_event(&mut self, event: MouseClickEvent) {
        if self.root_buddy.get_subscriptions().mouse_click {
            self.root_component.on_mouse_click(event, &mut self.root_buddy);
        }
    }
}

impl Drop for Application {

    fn drop(&mut self) {
        self.root_component.on_detach(&mut self.root_buddy);
    }
}