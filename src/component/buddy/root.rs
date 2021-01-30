use crate::*;
use std::cell::{Ref, RefCell};
use std::rc::Rc;

pub struct RootComponentBuddy {
    subscriptions: ComponentSubscriptions,

    // This is optional to ease the writing of unit tests, but the *Application* is expected to
    // call set_mouse_store in production environments.
    mouse_store: Option<Rc<RefCell<MouseStore>>>,

    last_render_result: Option<RenderResultStruct>,

    create_next_menu: Option<Box<dyn FnOnce(Box<dyn Component>) -> Box<dyn Component>>>,

    requested_render: bool,
}

impl RootComponentBuddy {
    pub fn new() -> Self {
        Self {
            subscriptions: ComponentSubscriptions::new(),
            mouse_store: None,
            last_render_result: None,
            create_next_menu: None,

            // Components should normally render as soon as possible after they
            // are attached
            requested_render: true,
        }
    }

    pub fn get_subscriptions(&self) -> &ComponentSubscriptions {
        &self.subscriptions
    }

    pub fn set_mouse_store(&mut self, mouse_store: Rc<RefCell<MouseStore>>) {
        self.mouse_store = Some(mouse_store);
    }

    fn get_mouse_store(&self) -> Ref<MouseStore> {
        self.mouse_store
            .as_ref()
            .expect("The application should use set_mouse_store")
            .borrow()
    }

    pub fn did_request_render(&self) -> bool {
        self.requested_render
    }

    pub fn get_last_render_result(&self) -> &Option<RenderResultStruct> {
        &self.last_render_result
    }

    pub fn set_last_render_result(&mut self, result: RenderResultStruct) {
        self.last_render_result = Some(result);
    }

    pub fn clear_render_request(&mut self) {
        self.requested_render = false;
    }

    pub fn has_next_menu(&self) -> bool {
        self.create_next_menu.is_some()
    }

    pub fn create_next_menu(&mut self, current_menu: Box<dyn Component>) -> Box<dyn Component> {
        let create_next_menu = self
            .create_next_menu
            .take()
            .expect("Only call this method after has_next_menu returned true");
        create_next_menu(current_menu)
    }
}

impl ComponentBuddy for RootComponentBuddy {
    fn change_menu(
        &mut self,
        create_new_menu: Box<dyn FnOnce(Box<dyn Component>) -> Box<dyn Component>>,
    ) {
        self.create_next_menu = Some(create_new_menu);
    }

    fn request_text_input(&self, start_text: String) -> Option<String> {
        unimplemented!()
    }

    fn request_render(&mut self) {
        self.requested_render = true;
    }

    fn subscribe_mouse_click(&mut self) {
        self.subscriptions.mouse_click = true;
    }

    fn unsubscribe_mouse_click(&mut self) {
        self.subscriptions.mouse_click = false;
    }

    fn subscribe_mouse_click_out(&mut self) {
        self.subscriptions.mouse_click_out = true;
    }

    fn unsubscribe_mouse_click_out(&mut self) {
        self.subscriptions.mouse_click_out = false;
    }

    fn subscribe_mouse_press(&mut self) {
        self.subscriptions.mouse_press = true;
    }

    fn unsubscribe_mouse_press(&mut self) {
        self.subscriptions.mouse_press = false;
    }

    fn subscribe_mouse_release(&mut self) {
        self.subscriptions.mouse_release = true;
    }

    fn unsubscribe_mouse_release(&mut self) {
        self.subscriptions.mouse_release = false;
    }

    fn subscribe_mouse_move(&mut self) {
        self.subscriptions.mouse_move = true;
    }

    fn unsubscribe_mouse_move(&mut self) {
        self.subscriptions.mouse_move = false;
    }

    fn subscribe_mouse_enter(&mut self) {
        self.subscriptions.mouse_enter = true;
    }

    fn unsubscribe_mouse_enter(&mut self) {
        self.subscriptions.mouse_enter = false;
    }

    fn subscribe_mouse_leave(&mut self) {
        self.subscriptions.mouse_leave = true;
    }

    fn unsubscribe_mouse_leave(&mut self) {
        self.subscriptions.mouse_leave = false;
    }

    fn subscribe_char_type(&self) -> Result<(), ()> {
        unimplemented!()
    }

    fn unsubscribe_char_type(&self) {
        unimplemented!()
    }

    fn get_mouse_position(&self, mouse: Mouse) -> Option<Point> {
        let mouse_store = self.get_mouse_store();
        // No transformation needed because we are the root
        mouse_store
            .get_mouse_state(mouse)
            .map(|state| state.position)
    }

    fn is_mouse_button_down(&self, mouse: Mouse, button: MouseButton) -> Option<bool> {
        let mouse_store = self.get_mouse_store();

        match mouse_store.get_mouse_state(mouse) {
            Some(state) => {
                if let Some(render_result) = &self.last_render_result {
                    if !render_result.filter_mouse_actions || render_result.drawn_region.is_inside(state.position) {
                        return Some(state.buttons.is_pressed(button));
                    }
                }
                None
            },
            None => None
        }
    }

    fn get_local_mouses(&self) -> Vec<Mouse> {
        let mouse_store = self.get_mouse_store();
        // No filtering needed since we are the root
        mouse_store.get_mouses()
    }

    fn get_all_mouses(&self) -> Vec<Mouse> {
        // All mouses are local for the root component
        self.get_local_mouses()
    }
}
