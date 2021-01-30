use crate::*;
use std::cell::RefCell;
use std::rc::Rc;

pub struct SimpleFlatBuddy {
    subscriptions: ComponentSubscriptions,

    mouse_buddy: Rc<RefCell<MouseBuddy>>,
    domain: ComponentDomain,

    last_render_result: Option<RenderResultStruct>,

    create_next_menu: Option<Box<dyn FnOnce(Box<dyn Component>) -> Box<dyn Component>>>,

    requested_render: bool,
    has_changes: bool,
}

impl SimpleFlatBuddy {
    pub(super) fn new(domain: ComponentDomain, mouse_buddy: Rc<RefCell<MouseBuddy>>) -> Self {
        Self {
            subscriptions: ComponentSubscriptions::new(),

            mouse_buddy,
            domain,

            last_render_result: None,
            create_next_menu: None,

            // Components should always render right after they are attached
            requested_render: true,
            // This one is initially true to indicate the requested_render
            has_changes: true,
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

    pub fn has_changes(&self) -> bool {
        self.has_changes
    }

    pub fn clear_changes(&mut self) {
        self.has_changes = false;
    }

    pub fn get_last_render_result(&self) -> &Option<RenderResultStruct> {
        &self.last_render_result
    }

    pub fn set_last_render_result(&mut self, result: RenderResultStruct) {
        self.last_render_result = Some(result);
    }

    pub fn has_next_menu(&self) -> bool {
        self.create_next_menu.is_some()
    }

    pub fn create_next_menu(
        &mut self,
    ) -> Box<dyn FnOnce(Box<dyn Component>) -> Box<dyn Component>> {
        self.create_next_menu
            .take()
            .expect("Only call this method after has_next_menu returned true")
    }
}

impl ComponentBuddy for SimpleFlatBuddy {
    fn change_menu(
        &mut self,
        create_new_menu: Box<dyn FnOnce(Box<dyn Component>) -> Box<dyn Component>>,
    ) {
        self.create_next_menu = Some(create_new_menu);
        self.has_changes = true;
    }

    fn request_text_input(&self, start_text: String) -> Option<String> {
        unimplemented!()
    }

    fn request_render(&mut self) {
        if !self.requested_render {
            self.requested_render = true;
            self.has_changes = true;
        }
    }

    fn subscribe_mouse_click(&mut self) {
        if !self.subscriptions.mouse_click {
            self.subscriptions.mouse_click = true;
            self.has_changes = true;
        }
    }

    fn unsubscribe_mouse_click(&mut self) {
        if self.subscriptions.mouse_click {
            self.subscriptions.mouse_click = false;
            self.has_changes = true;
        }
    }

    fn subscribe_mouse_click_out(&mut self) {
        if !self.subscriptions.mouse_click_out {
            self.subscriptions.mouse_click_out = true;
            self.has_changes = true;
        }
    }

    fn unsubscribe_mouse_click_out(&mut self) {
        if self.subscriptions.mouse_click_out {
            self.subscriptions.mouse_click_out = false;
            self.has_changes = true;
        }
    }

    fn subscribe_mouse_press(&mut self) {
        if !self.subscriptions.mouse_press {
            self.subscriptions.mouse_press = true;
            self.has_changes = true;
        }
    }

    fn unsubscribe_mouse_press(&mut self) {
        if self.subscriptions.mouse_press {
            self.subscriptions.mouse_press = false;
            self.has_changes = true;
        }
    }

    fn subscribe_mouse_release(&mut self) {
        if !self.subscriptions.mouse_release {
            self.subscriptions.mouse_release = true;
            self.has_changes = true;
        }
    }

    fn unsubscribe_mouse_release(&mut self) {
        if self.subscriptions.mouse_release {
            self.subscriptions.mouse_release = false;
            self.has_changes = true;
        }
    }

    fn subscribe_mouse_move(&mut self) {
        if !self.subscriptions.mouse_move {
            self.subscriptions.mouse_move = true;
            self.has_changes = true;
        }
    }

    fn unsubscribe_mouse_move(&mut self) {
        if self.subscriptions.mouse_move {
            self.subscriptions.mouse_move = false;
            self.has_changes = true;
        }
    }

    fn subscribe_mouse_enter(&mut self) {
        if !self.subscriptions.mouse_enter {
            self.subscriptions.mouse_enter = true;
            self.has_changes = true;
        }
    }

    fn unsubscribe_mouse_enter(&mut self) {
        if self.subscriptions.mouse_enter {
            self.subscriptions.mouse_enter = false;
            self.has_changes = true;
        }
    }

    fn subscribe_mouse_leave(&mut self) {
        if !self.subscriptions.mouse_leave {
            self.subscriptions.mouse_leave = true;
            self.has_changes = true;
        }
    }

    fn unsubscribe_mouse_leave(&mut self) {
        if self.subscriptions.mouse_leave {
            self.subscriptions.mouse_leave = false;
            self.has_changes = true;
        }
    }

    fn subscribe_char_type(&self) -> Result<(), ()> {
        unimplemented!()
    }

    fn unsubscribe_char_type(&self) {
        unimplemented!()
    }

    fn get_mouse_position(&self, mouse: Mouse) -> Option<Point> {
        let mouse_buddy = self.mouse_buddy.borrow();
        for entry in &mouse_buddy.local_mouses {
            if entry.mouse == mouse {
                return match self.domain.is_inside(entry.position) {
                    true => Some(self.domain.transform(entry.position)),
                    false => None,
                };
            }
        }
        None
    }

    fn is_mouse_button_down(&self, mouse: Mouse, button: MouseButton) -> Option<bool> {
        unimplemented!()
    }

    fn get_local_mouses(&self) -> Vec<Mouse> {
        let mouse_buddy = self.mouse_buddy.borrow();
        return mouse_buddy
            .local_mouses
            .iter()
            .filter(|mouse| self.domain.is_inside(mouse.position))
            .map(|mouse_entry| mouse_entry.mouse)
            .collect();
    }

    fn get_all_mouses(&self) -> Vec<Mouse> {
        let mouse_buddy = self.mouse_buddy.borrow();
        return mouse_buddy.all_mouses.clone();
    }
}

#[derive(Clone, Debug)]
pub(super) struct MouseBuddy {
    pub all_mouses: Vec<Mouse>,
    pub local_mouses: Vec<MouseEntry>,
}

#[derive(Copy, Clone, Debug)]
pub(super) struct MouseEntry {
    pub mouse: Mouse,
    pub position: Point,
}
