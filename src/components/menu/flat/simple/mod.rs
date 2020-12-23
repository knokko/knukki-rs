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
            let mut entry_to_add = ComponentEntry {
                component: to_add.component,
                domain: to_add.domain,
                buddy: SimpleFlatBuddy::new()
            };

            entry_to_add.attach();
            self.check_buddy(own_buddy, &mut entry_to_add);

            // Don't forget this x)
            self.components.push(Rc::new(RefCell::new(entry_to_add)));
        }
    }

    fn check_buddy(&self, own_buddy: &mut dyn ComponentBuddy, entry: &mut ComponentEntry) {
        if entry.buddy.has_changes() {
            if entry.buddy.did_request_render() {
                own_buddy.request_render();
                // Don't clear the render request until we have really rendered it
            }

            entry.buddy.clear_changes();
        }
    }

    fn get_component_at(&self, x: f32, y: f32) -> Option<RR<ComponentEntry>> {
        // TODO Performance: Use some kind of 2d range tree instead
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
        // TODO Performance improvement: only subscribe when at least 1 component did
        buddy.subscribe_mouse_click();
    }

    fn on_mouse_click(&mut self, event: MouseClickEvent, own_buddy: &mut dyn ComponentBuddy) {
        // This should be done before every important action
        self.update_internal(own_buddy);

        // Lets now handle the actual click event
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
        for entry_cell in &self.components {
            let mut entry = entry_cell.borrow_mut();
            entry.render();
        }
        // TODO Well... render the components x)
        // TODO And compute a more accurate return value!
        RenderResult::entire()
    }

    fn simulate_render(&mut self, region: RenderRegion, buddy: &mut dyn ComponentBuddy) -> RenderResult {
        // This needs to happen before each event
        self.update_internal(buddy);

        // Now onto the 'actual' drawing
        let mut drawn_regions: Vec<Box<dyn DrawnRegion>> = Vec::new();
        for entry_cell in &self.components {

            let mut entry = entry_cell.borrow_mut();
            let component_domain = entry.domain;
            let child_region = region.child_region(
                component_domain.get_min_x(),
                component_domain.get_min_y(),
                component_domain.get_max_x(),
                component_domain.get_max_y()
            );

            if let Some(entry_result) = entry.simulate_render(child_region) {
                let transformed_region = TransformedDrawnRegion::new(
                    entry_result.drawn_region.clone(),
                    move |x, y| component_domain.transform(x, y),
                    component_domain.get_min_x(),
                    component_domain.get_min_y(),
                    component_domain.get_max_x(),
                    component_domain.get_max_y()
                );
                drawn_regions.push(Box::new(transformed_region));
                self.check_buddy(buddy, &mut entry);
            }
        }

        RenderResult {
            drawn_region: Box::new(CompositeDrawnRegion::new(drawn_regions)),
            filter_mouse_actions: false
        }
    }

    fn on_detach(&mut self) {
        self.components.clear();
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
    fn attach(&mut self) {
        self.component.on_attach(&mut self.buddy);
    }

    fn mouse_click(&mut self, outer_event: MouseClickEvent) {
        if self.buddy.get_subscriptions().mouse_click {

            let transformed_point = self.domain.transform_mouse(outer_event.get_point());
            if let Some(render_result) = self.buddy.get_last_render_result() {
                if !render_result.filter_mouse_actions || 
                    render_result.drawn_region.is_mouse_inside(transformed_point) 
                {
                    let transformed_event = MouseClickEvent::new(
                        outer_event.get_mouse(), transformed_point, outer_event.get_button()
                    );

                    self.component.on_mouse_click(transformed_event, &mut self.buddy);
                }
            }
            
        }       
    }

    fn simulate_render(&mut self, region: RenderRegion) -> &Option<RenderResult> {
        if self.buddy.did_request_render() {
            let render_result = self.component.simulate_render(region, &mut self.buddy);
            self.buddy.set_last_render_result(render_result);
            self.buddy.get_last_render_result()
        } else {
            &None
        }
    }
}

impl Drop for ComponentEntry {
    fn drop(&mut self) {
        self.component.on_detach();
    }
}

#[cfg(test)]
mod tests {

    use crate::*;

    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn test_attach_and_detach() {
        struct CountingComponent {
            counter: Rc<Cell<u8>>
        }

        impl Component for CountingComponent {
            fn on_attach(&mut self, _buddy: &mut dyn ComponentBuddy) {
                self.counter.set(self.counter.get() + 1);
            }

            fn on_detach(&mut self) {
                self.counter.set(self.counter.get() + 1);
            }
        }

        let counter1 = Rc::new(Cell::new(0));
        let counter2 = Rc::new(Cell::new(0));

        let mut menu = SimpleFlatMenu::new();
        menu.add_component(
            Box::new(CountingComponent { counter: Rc::clone(&counter1) }), 
            ComponentDomain::between(0.0, 0.0, 0.5, 1.0)
        );

        let mut buddy = RootComponentBuddy::new();
        menu.on_attach(&mut buddy);

        // The first component should have been attached
        assert_eq!(1, counter1.get());
        assert_eq!(0, counter2.get());

        menu.add_component(
            Box::new(CountingComponent { counter: Rc::clone(&counter2) }), 
            ComponentDomain::between(0.5, 0.0, 1.0, 1.0)
        );

        // It should attach the second component as soon as possible
        menu.simulate_render(RenderRegion::between(0, 0, 10, 10), &mut buddy);
        assert_eq!(1, counter1.get());
        assert_eq!(1, counter2.get());

        // But they should be attached only once
        menu.simulate_render(RenderRegion::between(0, 0, 10, 10), &mut buddy);
        assert_eq!(1, counter1.get());
        assert_eq!(1, counter2.get());

        // When the menu is detached, so should their components be
        menu.on_detach();
        assert_eq!(2, counter1.get());
        assert_eq!(2, counter2.get());

        // And no 'second' detach when the menu is dropped
        drop(menu);
    }

    #[test]
    fn test_basic_mouse_clicking() {
        struct FullComponent {
            click_counter: Rc<Cell<u8>>
        }

        impl Component for FullComponent {
            fn on_attach(&mut self, buddy: &mut dyn ComponentBuddy) {
                buddy.subscribe_mouse_click();
            }

            fn on_mouse_click(&mut self, _event: MouseClickEvent, _buddy: &mut dyn ComponentBuddy) {
                self.click_counter.set(self.click_counter.get() + 1);
            }

            fn simulate_render(&mut self, _region: RenderRegion, _buddy: &mut dyn ComponentBuddy) -> RenderResult {
                RenderResult::entire()
            }
        }

        struct HalfComponent {
            click_counter: Rc<Cell<u8>>
        }

        impl Component for HalfComponent {
            fn on_attach(&mut self, buddy: &mut dyn ComponentBuddy) {
                buddy.subscribe_mouse_click();
            }

            fn on_mouse_click(&mut self, _event: MouseClickEvent, _buddy: &mut dyn ComponentBuddy) {
                self.click_counter.set(self.click_counter.get() + 1);
            }

            fn simulate_render(&mut self, _region: RenderRegion, _buddy: &mut dyn ComponentBuddy) -> RenderResult {
                RenderResult {
                    filter_mouse_actions: true,
                    drawn_region: Box::new(RectangularDrawnRegion::new(0.25, 0.0, 0.75, 1.0))
                }
            }
        }

        let full_counter = Rc::new(Cell::new(0));
        let half_counter = Rc::new(Cell::new(0));

        let mut menu = SimpleFlatMenu::new();
        menu.add_component(
            Box::new(FullComponent { click_counter: Rc::clone(&full_counter) }), 
            ComponentDomain::between(0.0, 0.0, 0.5, 0.5)
        );
        menu.add_component(
            Box::new(HalfComponent { click_counter: Rc::clone(&half_counter) }), 
            ComponentDomain::between(0.5, 0.5, 1.0, 1.0)
        );

        let mut application = Application::new(Box::new(menu));

        fn click_event(x: f32, y: f32) -> MouseClickEvent {
            MouseClickEvent::new(
                Mouse::new(0), 
                MousePoint::new(x, y), 
                MouseButton::primary()
            )
        }

        // Before the initial render, clicking shouldn't fire any events
        application.fire_mouse_click_event(click_event(0.2, 0.2));
        application.fire_mouse_click_event(click_event(0.7, 0.7));
        assert_eq!(0, full_counter.get());
        assert_eq!(0, half_counter.get());

        // After at least 1 render call, clicking should have effect
        application.simulate_render(RenderRegion::between(0, 0, 10, 10), false);
        application.fire_mouse_click_event(click_event(0.2, 0.2));
        application.fire_mouse_click_event(click_event(0.7, 0.7));
        assert_eq!(1, full_counter.get());
        assert_eq!(1, half_counter.get());

        // When clicking near the edge, only the full component should
        // receive the event
        application.fire_mouse_click_event(click_event(0.05, 0.05));
        application.fire_mouse_click_event(click_event(0.55, 0.55));
        assert_eq!(2, full_counter.get());
        assert_eq!(1, half_counter.get());

        // When miss-clicking entirely, neither should receive the event
        application.fire_mouse_click_event(click_event(0.1, 0.8));
        assert_eq!(2, full_counter.get());
        assert_eq!(1, half_counter.get());
    }
}