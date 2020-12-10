use crate::*;

pub struct RootComponentBuddy {

    subscriptions: ComponentSubscriptions,

    create_next_menu: Option<Box<dyn Fn(Box<dyn Component>) -> Box<dyn Component>>>,

    requested_render: bool
}

impl RootComponentBuddy {

    pub fn new() -> Self {
        Self {
            subscriptions: ComponentSubscriptions::new(),
            create_next_menu: None,
            requested_render: false
        }
    }

    pub fn get_subscriptions(&self) -> &ComponentSubscriptions {
        &self.subscriptions
    }

    pub fn did_request_render(&self) -> bool {
        self.requested_render
    }

    pub fn clear_render_request(&mut self) {
        self.requested_render = false;
    }

    pub fn has_next_menu(&self) -> bool {
        self.create_next_menu.is_some()
    }

    pub fn create_next_menu(&mut self, current_menu: Box<dyn Component>) -> Box<dyn Component> {
        let new_menu = self.create_next_menu.as_ref()
                .expect("Only call this method after has_next_menu returned true")(current_menu);
        self.create_next_menu = None;
        new_menu
    }
}

impl ComponentBuddy for RootComponentBuddy {
    
    fn change_menu(&self, create_new_menu: &dyn Fn(Box<dyn Component>) -> Box<dyn Component>) {
        unimplemented!()
    }

    fn request_text_input(&self, start_text: String) -> Option<String> {
        unimplemented!()
    }

    fn request_render(&mut self) {
        self.requested_render = true;
    }

    fn set_used_area(&self, area: Box<dyn ComponentArea>) {
        unimplemented!()
    }

    fn subscribe_mouse_click(&mut self) {
        self.subscriptions.mouse_click = true;
    }

    fn unsubscribe_mouse_click(&mut self) {
        self.subscriptions.mouse_click = false;
    }

    fn subscribe_mouse_click_out(&self) {
        unimplemented!()
    }

    fn unsubscribe_mouse_click_out(&self) {
        unimplemented!()
    }

    fn subscribe_mouse_move(&self) {
        unimplemented!()
    }

    fn unsubscribe_mouse_move(&self) {
        unimplemented!()
    }

    fn subscribe_mouse_enter(&self) {
        unimplemented!()
    }

    fn unsubscribe_mouse_enter(&self) {
        unimplemented!()
    }

    fn subscribe_mouse_leave(&self) {
        unimplemented!()
    }

    fn unsubscribe_mouse_leave(&self) {
        unimplemented!()
    }

    fn subscribe_char_type(&self) -> Result<(), ()> {
        unimplemented!()
    }

    fn unsubscribe_char_type(&self) {
        unimplemented!()
    }

    fn get_mouse_position(&self, mouse: Mouse) -> Option<MousePoint> {
        unimplemented!()
    }

    fn is_mouse_down(&self, mouse: Mouse, button: MouseButton) -> bool {
        unimplemented!()
    }

    fn is_primary_mouse_down(&self, mouse: Mouse) -> bool {
        unimplemented!()
    }

    fn get_local_mouses(&self) -> Vec<Mouse> {
        unimplemented!()
    }

    fn get_all_mouses(&self) -> Vec<Mouse> {
        unimplemented!()
    }

    fn get_aspect_ratio(&self) -> f32 {
        unimplemented!()
    }
}