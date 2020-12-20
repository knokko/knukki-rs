use crate::*;

use std::cell::RefCell;
use std::rc::{
    Rc,
    Weak
};

mod buddy;
mod domain;

pub use domain::*;
use buddy::*;

type RR<T> = Rc<RefCell<T>>;
type WR<T> = Weak<RefCell<T>>;

pub struct SimpleFlatMenu {

    components: Vec<RR<ComponentEntry>>,
    components_to_add: Vec<ComponentToAdd>
}

impl SimpleFlatMenu {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            components_to_add: Vec::new()
        }
    }

    pub fn add_component(&mut self, component: Box<dyn Component>, domain: ComponentDomain) {
        self.components_to_add.push(ComponentToAdd { component, domain });
    }

    fn update_internal(&mut self, own_buddy: &mut dyn ComponentBuddy) {
        while !self.components_to_add.is_empty() {
            let to_add = self.components_to_add.swap_remove(0);
            let entry_to_add = ComponentEntry {
                component: to_add.component,
                domain: to_add.domain,
                buddy: SimpleFlatBuddy::new()
            };
            // TODO Create buddy and attach the component

            // For this part, it is important that subscribing twice is equivalent
            // to subscribing once.
            own_buddy.request_render();

            // Don't forget this x)
            self.components.push(Rc::new(RefCell::new(entry_to_add)));
        }
    }

    fn check_buddy(&mut self, own_buddy: &mut dyn ComponentBuddy, entry: &mut ComponentEntry) {
        if entry.buddy.has_changes() {
            if entry.buddy.get_subscriptions().mouse_click {
                own_buddy.subscribe_mouse_click();
            } else {
                own_buddy.unsubscribe_mouse_click();
            }
            
            if entry.buddy.did_request_render() {
                own_buddy.request_render();
                // Don't clear the render request until we have really rendered it
            }

            entry.buddy.clear_changes();
        }
    }

    fn get_component_at(&self, x: f32, y: f32) -> Option<RR<ComponentEntry>> {
        for entry_cell in &self.components {
            let entry = entry_cell.borrow();
            if entry.domain.is_inside(x, y) {
                return Some(Rc::clone(&entry_cell));
            }
        }

        None
    }
}

impl Component for SimpleFlatMenu {
    fn on_attach(&mut self, buddy: &mut dyn ComponentBuddy) {
        self.update_internal(buddy);
    }

    fn on_mouse_click(&mut self, event: MouseClickEvent, own_buddy: &mut dyn ComponentBuddy) {
        let maybe_clicked_cell = self.get_component_at(
            event.get_point().get_x(), event.get_point().get_y()
        );

        if let Some(clicked_cell) = maybe_clicked_cell {
            let mut clicked_entry = clicked_cell.borrow_mut();
            clicked_entry.mouse_click(event);
            self.check_buddy(own_buddy, &mut clicked_entry);
        }

        // TODO Fire mouse click out events to the rest of the components
    }

    #[cfg(feature = "golem_rendering")]
    fn render(&mut self, golem: &golem::Context, region: RenderRegion, buddy: &mut dyn ComponentBuddy) -> RenderResult {
        // TODO Well... render the components x)
        // TODO And compute a more accurate return value!
        RenderResult::entire()
    }
}

struct ComponentToAdd {
    component: Box<dyn Component>,
    domain: ComponentDomain
}

struct ComponentEntry {
    component: Box<dyn Component>,
    domain: ComponentDomain,
    buddy: SimpleFlatBuddy,
}

impl ComponentEntry {
    fn mouse_click(&mut self, outer_event: MouseClickEvent) {
        if self.buddy.get_subscriptions().mouse_click {

            let transformed_point = self.domain.transform_mouse(outer_event.get_point());
            let transformed_event = MouseClickEvent::new(
                outer_event.get_mouse(), transformed_point, outer_event.get_button()
            );

            self.component.on_mouse_click(transformed_event, &mut self.buddy);
        }       
    }
}
