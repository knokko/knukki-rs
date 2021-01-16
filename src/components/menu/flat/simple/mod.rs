use crate::*;

use std::cell::RefCell;
use std::rc::{Rc, Weak};

mod buddy;
mod domain;

use buddy::*;
pub use domain::*;

type RR<T> = Rc<RefCell<T>>;
type WR<T> = Weak<RefCell<T>>;

pub struct SimpleFlatMenu {
    components: Vec<RR<ComponentEntry>>,
    components_to_add: Vec<ComponentToAdd>,
    background_color: Option<Color>,
    has_rendered_before: bool,

    mouse_buddy: RR<MouseBuddy>,
}

impl SimpleFlatMenu {
    pub fn new(background_color: Option<Color>) -> Self {
        Self {
            components: Vec::new(),
            components_to_add: Vec::new(),
            background_color,
            has_rendered_before: false,

            mouse_buddy: Rc::new(RefCell::new(MouseBuddy {
                all_mouses: Vec::new(),
                local_mouses: Vec::new(),
            })),
        }
    }

    pub fn add_component(&mut self, component: Box<dyn Component>, domain: ComponentDomain) {
        self.components_to_add
            .push(ComponentToAdd { component, domain });
    }

    fn update_internal(&mut self, own_buddy: &mut dyn ComponentBuddy, is_about_to_render: bool) {
        while !self.components_to_add.is_empty() {
            let to_add = self.components_to_add.swap_remove(0);
            let mut entry_to_add = ComponentEntry {
                component: to_add.component,
                domain: to_add.domain,
                buddy: SimpleFlatBuddy::new(to_add.domain, Rc::clone(&self.mouse_buddy)),
            };

            entry_to_add.attach();
            self.check_buddy(own_buddy, &mut entry_to_add, is_about_to_render);

            // Don't forget this x)
            self.components.push(Rc::new(RefCell::new(entry_to_add)));
        }

        // Keep the mouse buddy up-to-date
        let mut mouse_buddy = self.mouse_buddy.borrow_mut();
        mouse_buddy.all_mouses = own_buddy.get_all_mouses();
        let local_mouses = own_buddy.get_local_mouses();
        mouse_buddy.local_mouses.clear();
        for mouse in local_mouses {
            let should_have_position = own_buddy.get_mouse_position(mouse);
            if let Some(position) = should_have_position {
                mouse_buddy.local_mouses.push(MouseEntry {
                    mouse,
                    position
                });
            } else {
                // This is weird behavior that should be investigated, but not worth a production
                // crash
                debug_assert!(false);
            }
        }
        drop(mouse_buddy);
    }

    fn check_buddy(
        &self,
        own_buddy: &mut dyn ComponentBuddy,
        entry: &mut ComponentEntry,
        is_about_to_render: bool,
    ) {
        if entry.buddy.has_changes() {
            if !is_about_to_render && entry.buddy.did_request_render() {
                own_buddy.request_render();
                // Don't clear the render request until we have really rendered it
            }

            entry.buddy.clear_changes();
        }
    }

    fn get_component_at(&self, point: Point) -> Option<RR<ComponentEntry>> {
        // TODO Performance: Use some kind of 2d range tree instead
        for entry_cell in &self.components {
            let entry = entry_cell.borrow();
            if entry.domain.is_inside(point) {
                return Some(Rc::clone(&entry_cell));
            }
        }

        None
    }
}

impl Component for SimpleFlatMenu {
    fn on_attach(&mut self, buddy: &mut dyn ComponentBuddy) {
        self.update_internal(buddy, false);
        buddy.subscribe_mouse_click();
        buddy.subscribe_mouse_click_out();
        buddy.subscribe_mouse_move();
        buddy.subscribe_mouse_enter();
        buddy.subscribe_mouse_leave();
    }

    // Variables only used when the golem_rendering feature is enabled are
    // considered 'unused' when compiling without this feature.
    #[allow(unused_variables)]
    fn render(
        &mut self,
        #[cfg(feature = "golem_rendering")] golem: &golem::Context,
        region: RenderRegion,
        buddy: &mut dyn ComponentBuddy,
        force: bool,
    ) -> RenderResult {
        // This needs to happen before each event
        self.update_internal(buddy, true);

        // Now onto the 'actual' drawing
        if force || !self.has_rendered_before {
            if let Some(bc) = self.background_color {
                // TODO And take more care when this is partially transparent...
                #[cfg(feature = "golem_rendering")]
                golem.set_clear_color(
                    bc.get_red_float(),
                    bc.get_green_float(),
                    bc.get_blue_float(),
                    bc.get_alpha_float(),
                );
                #[cfg(feature = "golem_rendering")]
                golem.clear();
            }
        }
        let mut drawn_regions: Vec<Box<dyn DrawnRegion>> = Vec::new();
        for entry_cell in &self.components {
            let mut entry = entry_cell.borrow_mut();
            let component_domain = entry.domain;
            let child_region = region.child_region(
                component_domain.get_min_x(),
                component_domain.get_min_y(),
                component_domain.get_max_x(),
                component_domain.get_max_y(),
            );

            if let Some(entry_result) = entry.render(
                #[cfg(feature = "golem_rendering")]
                golem,
                child_region,
                force,
            ) {
                match entry_result {
                    Ok(good_entry_result) => {
                        let transformed_region = TransformedDrawnRegion::new(
                            good_entry_result.drawn_region.clone(),
                            move |point| component_domain.transform(point),
                            move |point| component_domain.transform_back(point),
                        );
                        if !force || self.background_color.is_none() {
                            drawn_regions.push(Box::new(transformed_region));
                        }
                        self.check_buddy(buddy, &mut entry, false);
                    }
                    Err(bad_result) => {
                        return Err(bad_result);
                    }
                }
            }
        }

        if (force || !self.has_rendered_before) && self.background_color.is_some() {
            self.has_rendered_before = true;
            entire_render_result()
        } else {
            self.has_rendered_before = true;
            Ok(RenderResultStruct {
                drawn_region: Box::new(CompositeDrawnRegion::new(drawn_regions)),
                filter_mouse_actions: false,
            })
        }
    }

    fn on_mouse_click(&mut self, event: MouseClickEvent, own_buddy: &mut dyn ComponentBuddy) {
        // This should be done before every important action
        self.update_internal(own_buddy, false);

        // Lets now handle the actual click event
        let maybe_clicked_cell = self.get_component_at(event.get_point());

        if let Some(clicked_cell) = &maybe_clicked_cell {
            let mut clicked_entry = clicked_cell.borrow_mut();
            clicked_entry.mouse_click(event);
            self.check_buddy(own_buddy, &mut clicked_entry, false);
        }

        // TODO Maintain a list for just the interested components
        let out_event = MouseClickOutEvent::new(event.get_mouse(), event.get_button());
        for component_cell in &self.components {
            if maybe_clicked_cell.is_none()
                || !Rc::ptr_eq(component_cell, maybe_clicked_cell.as_ref().unwrap())
            {
                let mut component_entry = component_cell.borrow_mut();
                component_entry.mouse_click_out(out_event);
                self.check_buddy(own_buddy, &mut component_entry, false);
            }
        }
    }

    fn on_mouse_click_out(
        &mut self,
        event: MouseClickOutEvent,
        own_buddy: &mut dyn ComponentBuddy,
    ) {
        // TODO Maintain a list for just the interested components
        for component_cell in &self.components {
            let mut component_entry = component_cell.borrow_mut();
            component_entry.mouse_click_out(event);
            self.check_buddy(own_buddy, &mut component_entry, false);
        }
    }

    fn on_mouse_move(&mut self, event: MouseMoveEvent, buddy: &mut dyn ComponentBuddy) {
        // TODO Consider only the components intersecting the rectangle around the line from
        // TODO event.from to event.to (using some kind of 2d range tree)
        for entry_cell in &self.components {
            let mut entry = entry_cell.borrow_mut();
            entry.mouse_move(event);
            self.check_buddy(buddy, &mut entry, false);
        }
    }

    fn on_mouse_enter(&mut self, event: MouseEnterEvent, buddy: &mut dyn ComponentBuddy) {
        if let Some(hit_component_entry) = self.get_component_at(event.get_entrance_point()) {
            let mut borrowed_entry = hit_component_entry.borrow_mut();
            borrowed_entry.mouse_enter(event);
            self.check_buddy(buddy, &mut borrowed_entry, false);
        }
    }

    fn on_mouse_leave(&mut self, event: MouseLeaveEvent, buddy: &mut dyn ComponentBuddy) {
        if let Some(hit_component_entry) = self.get_component_at(event.get_exit_point()) {
            let mut borrowed_entry = hit_component_entry.borrow_mut();
            borrowed_entry.mouse_leave(event);
            self.check_buddy(buddy, &mut borrowed_entry, false);
        }
    }

    fn on_detach(&mut self) {
        self.components.clear();
    }
}

struct ComponentToAdd {
    component: Box<dyn Component>,
    domain: ComponentDomain,
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
        let mut filtered = false;
        if self.buddy.get_subscriptions().mouse_click {
            let transformed_point = self.domain.transform(outer_event.get_point());
            if let Some(render_result) = self.buddy.get_last_render_result() {
                if !render_result.filter_mouse_actions
                    || render_result.drawn_region.is_inside(transformed_point)
                {
                    let transformed_event = MouseClickEvent::new(
                        outer_event.get_mouse(),
                        transformed_point,
                        outer_event.get_button(),
                    );

                    self.component
                        .on_mouse_click(transformed_event, &mut self.buddy);
                } else {
                    filtered = true;
                }
            }
        }

        if filtered && self.buddy.get_subscriptions().mouse_click_out {
            self.component.on_mouse_click_out(
                MouseClickOutEvent::new(outer_event.get_mouse(), outer_event.get_button()),
                &mut self.buddy,
            );
        }
    }

    fn mouse_click_out(&mut self, event: MouseClickOutEvent) {
        if self.buddy.get_subscriptions().mouse_click_out {
            if self.buddy.get_last_render_result().is_some() {
                self.component.on_mouse_click_out(event, &mut self.buddy);
            }
        }
    }

    fn mouse_enter(&mut self, event: MouseEnterEvent) {
        if self.buddy.get_subscriptions().mouse_enter {
            if let Some(render_result) = self.buddy.get_last_render_result() {
                let transformed_entrance_point = self.domain.transform(event.get_entrance_point());
                if !render_result.filter_mouse_actions || render_result.drawn_region.is_inside(transformed_entrance_point) {
                    let transformed_event = MouseEnterEvent::new(
                        event.get_mouse(), transformed_entrance_point
                    );
                    self.component.on_mouse_enter(transformed_event, &mut self.buddy);
                }
            }
        }
    }

    fn mouse_leave(&mut self, event: MouseLeaveEvent) {
        if self.buddy.get_subscriptions().mouse_leave {
            if let Some(render_result) = self.buddy.get_last_render_result() {
                let transformed_exit_point = self.domain.transform(event.get_exit_point());
                if !render_result.filter_mouse_actions || render_result.drawn_region.is_inside(transformed_exit_point) {
                    let transformed_event = MouseLeaveEvent::new(
                        event.get_mouse(), transformed_exit_point
                    );
                    self.component.on_mouse_leave(transformed_event, &mut self.buddy);
                }
            }
        }
    }

    fn mouse_move(&mut self, event: MouseMoveEvent) {
        let sub_enter = self.buddy.get_subscriptions().mouse_enter;
        let sub_move = self.buddy.get_subscriptions().mouse_move;
        let sub_leave = self.buddy.get_subscriptions().mouse_leave;
        if sub_enter || sub_move || sub_leave {
            if let Some(render_result) = self.buddy.get_last_render_result() {
                let transformed_from = self.domain.transform(event.get_from());
                let transformed_to = self.domain.transform(event.get_to());
                let backup_region = RectangularDrawnRegion::new(
                    0.0, 0.0, 1.0, 1.0
                );
                let reference_region = match render_result.filter_mouse_actions {
                    true => render_result.drawn_region.as_ref(),
                    false => &backup_region
                };
                let intersection = reference_region.find_line_intersection(transformed_from, transformed_to);
                match intersection {
                    LineIntersection::FullyOutside => {
                        // I don't need to do anything
                    }, LineIntersection::FullyInside => {
                        // Just pass a MouseMoveEvent
                        if sub_move {
                            let move_event = MouseMoveEvent::new(
                                event.get_mouse(), transformed_from, transformed_to
                            );
                            self.component.on_mouse_move(move_event, &mut self.buddy);
                        }
                    }, LineIntersection::Enters { point } => {
                        // Pass a MouseEnterEvent and a MouseMoveEvent
                        if sub_enter {
                            let enter_event = MouseEnterEvent::new(
                                event.get_mouse(), point
                            );
                            self.component.on_mouse_enter(enter_event, &mut self.buddy);
                        }

                        // Note: the component might have subscribed during its on_mouse_enter
                        if self.buddy.get_subscriptions().mouse_move {
                            let move_event = MouseMoveEvent::new(
                                event.get_mouse(), point, transformed_to
                            );
                            self.component.on_mouse_move(move_event, &mut self.buddy);
                        }
                    }, LineIntersection::Exits { point } => {
                        // Pass a MouseMoveEvent and a MouseLeaveEvent
                        if sub_move {
                            let move_event = MouseMoveEvent::new(
                                event.get_mouse(), transformed_from, point
                            );
                            self.component.on_mouse_move(move_event, &mut self.buddy);
                        }

                        // Note: the component might have subscribed during its on_mouse_move
                        if self.buddy.get_subscriptions().mouse_leave {
                            let leave_event = MouseLeaveEvent::new(
                                event.get_mouse(), point
                            );
                            self.component.on_mouse_leave(leave_event, &mut self.buddy);
                        }
                    }, LineIntersection::Crosses { entrance, exit } => {
                        // Pass a MouseEnterEvent, MouseMoveEvent, and MouseLeaveEvent
                        if sub_enter {
                            let enter_event = MouseEnterEvent::new(
                                event.get_mouse(), entrance
                            );
                            self.component.on_mouse_enter(enter_event, &mut self.buddy);
                        }

                        // Note: the component might have subscribed during its on_mouse_enter
                        if self.buddy.get_subscriptions().mouse_move {
                            let move_event = MouseMoveEvent::new(
                                event.get_mouse(), entrance, exit
                            );
                            self.component.on_mouse_move(move_event, &mut self.buddy);
                        }

                        if self.buddy.get_subscriptions().mouse_leave {
                            let leave_event = MouseLeaveEvent::new(
                                event.get_mouse(), exit
                            );
                            self.component.on_mouse_leave(leave_event, &mut self.buddy);
                        }
                    }
                };
            }
        }
    }

    fn render(
        &mut self,
        #[cfg(feature = "golem_rendering")] golem: &golem::Context,
        region: RenderRegion,
        force: bool,
    ) -> Option<RenderResult> {
        if force || self.buddy.did_request_render() {
            self.buddy.clear_render_request();
            #[cfg(feature = "golem_rendering")]
            {
                region.set_viewport(golem);
                region.set_scissor(golem);
            }

            let render_result = self.component.render(
                #[cfg(feature = "golem_rendering")]
                golem,
                region,
                &mut self.buddy,
                force,
            );
            if render_result.is_err() {
                return Some(render_result);
            }

            let good_result = render_result.unwrap();
            self.buddy.set_last_render_result(good_result.clone());
            Some(Ok(good_result))
        } else {
            None
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

    use std::cell::*;
    use std::rc::Rc;

    fn root_buddy() -> RootComponentBuddy {
        let mut buddy = RootComponentBuddy::new();
        init(&mut buddy);
        buddy
    }

    fn init(buddy: &mut RootComponentBuddy) {
        buddy.set_mouse_store(Rc::new(RefCell::new(MouseStore::new())));
    }

    #[test]
    fn test_attach_and_detach() {
        struct CountingComponent {
            counter: Rc<Cell<u8>>,
        }

        impl Component for CountingComponent {
            fn on_attach(&mut self, _buddy: &mut dyn ComponentBuddy) {
                self.counter.set(self.counter.get() + 1);
            }

            fn render(
                &mut self,
                _region: RenderRegion,
                _buddy: &mut dyn ComponentBuddy,
                _force: bool,
            ) -> RenderResult {
                entire_render_result()
            }

            fn on_detach(&mut self) {
                self.counter.set(self.counter.get() + 1);
            }
        }

        let counter1 = Rc::new(Cell::new(0));
        let counter2 = Rc::new(Cell::new(0));

        let mut menu = SimpleFlatMenu::new(None);
        menu.add_component(
            Box::new(CountingComponent {
                counter: Rc::clone(&counter1),
            }),
            ComponentDomain::between(0.0, 0.0, 0.5, 1.0),
        );

        let mut buddy = root_buddy();
        menu.on_attach(&mut buddy);

        // The first component should have been attached
        assert_eq!(1, counter1.get());
        assert_eq!(0, counter2.get());

        menu.add_component(
            Box::new(CountingComponent {
                counter: Rc::clone(&counter2),
            }),
            ComponentDomain::between(0.5, 0.0, 1.0, 1.0),
        );

        // It should attach the second component as soon as possible
        menu.render(RenderRegion::between(0, 0, 10, 10), &mut buddy, false)
            .unwrap();
        assert_eq!(1, counter1.get());
        assert_eq!(1, counter2.get());

        // But they should be attached only once
        menu.render(RenderRegion::between(0, 0, 10, 10), &mut buddy, false)
            .unwrap();
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
            click_counter: Rc<Cell<u8>>,
        }

        impl Component for FullComponent {
            fn on_attach(&mut self, buddy: &mut dyn ComponentBuddy) {
                buddy.subscribe_mouse_click();
            }

            fn on_mouse_click(&mut self, _event: MouseClickEvent, _buddy: &mut dyn ComponentBuddy) {
                self.click_counter.set(self.click_counter.get() + 1);
            }

            fn render(
                &mut self,
                _region: RenderRegion,
                _buddy: &mut dyn ComponentBuddy,
                _force: bool,
            ) -> RenderResult {
                entire_render_result()
            }
        }

        struct HalfComponent {
            click_counter: Rc<Cell<u8>>,
        }

        impl Component for HalfComponent {
            fn on_attach(&mut self, buddy: &mut dyn ComponentBuddy) {
                buddy.subscribe_mouse_click();
            }

            fn on_mouse_click(&mut self, _event: MouseClickEvent, _buddy: &mut dyn ComponentBuddy) {
                self.click_counter.set(self.click_counter.get() + 1);
            }

            fn render(
                &mut self,
                _region: RenderRegion,
                _buddy: &mut dyn ComponentBuddy,
                _force: bool,
            ) -> RenderResult {
                Ok(RenderResultStruct {
                    filter_mouse_actions: true,
                    drawn_region: Box::new(RectangularDrawnRegion::new(0.25, 0.0, 0.75, 1.0)),
                })
            }
        }

        let full_counter = Rc::new(Cell::new(0));
        let half_counter = Rc::new(Cell::new(0));

        let mut menu = SimpleFlatMenu::new(None);
        menu.add_component(
            Box::new(FullComponent {
                click_counter: Rc::clone(&full_counter),
            }),
            ComponentDomain::between(0.0, 0.0, 0.5, 0.5),
        );
        menu.add_component(
            Box::new(HalfComponent {
                click_counter: Rc::clone(&half_counter),
            }),
            ComponentDomain::between(0.5, 0.5, 1.0, 1.0),
        );

        let mut application = Application::new(Box::new(menu));

        fn click_event(x: f32, y: f32) -> MouseClickEvent {
            MouseClickEvent::new(Mouse::new(0), Point::new(x, y), MouseButton::primary())
        }

        // Before the initial render, clicking shouldn't fire any events
        application.fire_mouse_click_event(click_event(0.2, 0.2));
        application.fire_mouse_click_event(click_event(0.7, 0.7));
        assert_eq!(0, full_counter.get());
        assert_eq!(0, half_counter.get());

        // After at least 1 render call, clicking should have effect
        application.render(RenderRegion::between(0, 0, 10, 10), false);
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

    #[test]
    fn test_rendering_components() {
        struct BusyRenderComponent {
            counter: Rc<Cell<u8>>,
        }

        impl Component for BusyRenderComponent {
            fn on_attach(&mut self, _buddy: &mut dyn ComponentBuddy) {}

            fn render(
                &mut self,
                _region: RenderRegion,
                buddy: &mut dyn ComponentBuddy,
                _force: bool,
            ) -> RenderResult {
                buddy.request_render();
                self.counter.set(self.counter.get() + 1);
                entire_render_result()
            }
        }

        struct ClickRenderComponent {
            counter: Rc<Cell<u8>>,
        }

        impl Component for ClickRenderComponent {
            fn on_attach(&mut self, buddy: &mut dyn ComponentBuddy) {
                buddy.subscribe_mouse_click();
            }

            fn on_mouse_click(&mut self, _event: MouseClickEvent, buddy: &mut dyn ComponentBuddy) {
                buddy.request_render();
            }

            fn render(
                &mut self,
                _region: RenderRegion,
                _buddy: &mut dyn ComponentBuddy,
                _force: bool,
            ) -> RenderResult {
                self.counter.set(self.counter.get() + 1);
                entire_render_result()
            }
        }

        let mut menu = SimpleFlatMenu::new(None);
        let mut buddy = root_buddy();
        let render_region = RenderRegion::between(10, 20, 30, 40);

        let click_counter = Rc::new(Cell::new(0));
        let busy_counter = Rc::new(Cell::new(0));

        menu.add_component(
            Box::new(ClickRenderComponent {
                counter: Rc::clone(&click_counter),
            }),
            ComponentDomain::between(0.1, 0.1, 0.3, 0.3),
        );

        // All components should be rendered during their first render call
        assert!(buddy.did_request_render());
        buddy.clear_render_request();
        menu.render(render_region, &mut buddy, false).unwrap();
        assert_eq!(1, click_counter.get());

        // But the menu shouldn't request another one until we click the component
        assert!(!buddy.did_request_render());

        // So let's click it and render again
        let hit_click =
            MouseClickEvent::new(Mouse::new(0), Point::new(0.2, 0.2), MouseButton::primary());
        menu.on_mouse_click(hit_click, &mut buddy);
        assert!(buddy.did_request_render());
        buddy.clear_render_request();
        menu.render(render_region, &mut buddy, false).unwrap();
        assert!(!buddy.did_request_render());
        assert_eq!(2, click_counter.get());

        // Miss clicking shouldn't change anything
        let miss_click = MouseClickEvent::new(
            Mouse::new(0),
            Point::new(0.35, 0.35),
            MouseButton::primary(),
        );
        menu.on_mouse_click(miss_click, &mut buddy);
        assert!(!buddy.did_request_render());

        // Force rendering should cause the render method to be called
        // And thus increment the click counter
        buddy.clear_render_request();
        menu.render(render_region, &mut buddy, true).unwrap();
        assert!(!buddy.did_request_render());
        assert_eq!(3, click_counter.get());

        // Add the busy rendering component
        menu.add_component(
            Box::new(BusyRenderComponent {
                counter: Rc::clone(&busy_counter),
            }),
            ComponentDomain::between(0.5, 0.7, 0.9, 0.8),
        );

        // Only the busy component should render
        buddy.clear_render_request();
        menu.render(render_region, &mut buddy, false).unwrap();
        // And it should have requested a redraw right away
        assert!(buddy.did_request_render());
        assert_eq!(3, click_counter.get());
        assert_eq!(1, busy_counter.get());

        // Again, only the busy component should render
        buddy.clear_render_request();
        menu.render(render_region, &mut buddy, false).unwrap();
        // And it should have requested a redraw again
        assert!(buddy.did_request_render());
        assert_eq!(3, click_counter.get());
        assert_eq!(2, busy_counter.get());

        // When we force a render, both components should be redrawn
        buddy.clear_render_request();
        menu.render(render_region, &mut buddy, true).unwrap();
        // Like usual, it should request a next render
        assert!(buddy.did_request_render());
        assert_eq!(4, click_counter.get());
        assert_eq!(3, busy_counter.get());

        // But when we render without force again, only the busy one should be drawn
        buddy.clear_render_request();
        menu.render(render_region, &mut buddy, false).unwrap();
        assert!(buddy.did_request_render());
        assert_eq!(4, click_counter.get());
        assert_eq!(4, busy_counter.get());

        // Unless we click it...
        menu.on_mouse_click(hit_click, &mut buddy);
        menu.render(render_region, &mut buddy, false).unwrap();
        assert!(buddy.did_request_render());
        assert_eq!(5, click_counter.get());
        assert_eq!(5, busy_counter.get());
    }

    struct ClickComponent {
        render_result: RenderResult,
    }

    impl Component for ClickComponent {
        fn on_attach(&mut self, buddy: &mut dyn ComponentBuddy) {
            buddy.subscribe_mouse_click();
        }

        fn render(
            &mut self,
            _region: RenderRegion,
            _buddy: &mut dyn ComponentBuddy,
            _force: bool,
        ) -> RenderResult {
            self.render_result.clone()
        }

        fn on_mouse_click(&mut self, _event: MouseClickEvent, buddy: &mut dyn ComponentBuddy) {
            buddy.request_render();
        }
    }

    #[test]
    fn test_render_results_with_background() {
        test_render_results(Some(Color::rgb(0, 200, 100)), true);
    }

    #[test]
    fn test_render_results_without_background() {
        test_render_results(None, false);
    }

    fn test_render_results(background: Option<Color>, draw_background: bool) {
        let mut menu = SimpleFlatMenu::new(background);
        menu.add_component(
            Box::new(ClickComponent {
                render_result: entire_render_result(),
            }),
            ComponentDomain::between(0.0, 0.0, 0.5, 0.5),
        );
        menu.add_component(
            Box::new(ClickComponent {
                render_result: Ok(RenderResultStruct {
                    drawn_region: Box::new(RectangularDrawnRegion::new(0.0, 0.0, 0.6, 0.6)),
                    filter_mouse_actions: false,
                }),
            }),
            ComponentDomain::between(0.5, 0.5, 1.0, 1.0),
        );

        let click1 =
            MouseClickEvent::new(Mouse::new(0), Point::new(0.2, 0.2), MouseButton::primary());
        let click2 =
            MouseClickEvent::new(Mouse::new(0), Point::new(0.6, 0.6), MouseButton::primary());
        let render_region = RenderRegion::between(0, 0, 100, 40);
        let mut buddy = root_buddy();

        let result = menu.render(render_region, &mut buddy, false).unwrap();
        // This menu should never request filtering mouse actions
        assert!(!result.filter_mouse_actions);
        // It should have rendered both components, and maybe the area outside
        {
            let region = &result.drawn_region;
            assert_eq!(draw_background, region.is_inside(Point::new(1.0, 0.0)));
            assert_eq!(draw_background, region.is_inside(Point::new(0.0, 1.0)));
            assert!(region.is_inside(Point::new(0.0, 0.0)));
            assert!(region.is_inside(Point::new(0.5, 0.5)));
            assert!(region.is_inside(Point::new(0.7, 0.7)));
            // Only draw outside the drawn region of component 2 if there is background
            assert_eq!(draw_background, region.is_inside(Point::new(0.9, 0.9)));
        }

        menu.on_mouse_click(click1, &mut buddy);
        let result = menu.render(render_region, &mut buddy, false).unwrap();
        // This time, it should only have drawn the first component
        {
            let region = &result.drawn_region;
            assert!(region.is_inside(Point::new(0.0, 0.0)));
            assert!(region.is_inside(Point::new(0.5, 0.5)));
            assert!(!region.is_inside(Point::new(0.6, 0.6)));
        }

        menu.on_mouse_click(click2, &mut buddy);
        let result = menu.render(render_region, &mut buddy, false).unwrap();
        // This time, it should only have drawn the second component
        {
            let region = &result.drawn_region;
            assert!(!region.is_inside(Point::new(0.2, 0.2)));
            assert!(region.is_inside(Point::new(0.5, 0.5)));
            assert!(region.is_inside(Point::new(0.7, 0.7)));
            // But only inside the region returned by the second component
            assert!(!region.is_inside(Point::new(0.9, 0.9)));
        }

        // When we force, it should draw both components again, and maybe even background
        let result = menu.render(render_region, &mut buddy, true).unwrap();
        {
            let region = &result.drawn_region;
            assert_eq!(draw_background, region.is_inside(Point::new(1.0, 0.0)));
            assert_eq!(draw_background, region.is_inside(Point::new(0.0, 1.0)));
            assert!(region.is_inside(Point::new(0.0, 0.0)));
            assert!(region.is_inside(Point::new(0.5, 0.5)));
            assert!(region.is_inside(Point::new(0.7, 0.7)));
            // Only draw outside the drawn region of component 2 if there is background
            assert_eq!(draw_background, region.is_inside(Point::new(0.9, 0.9)));
        }

        // And when we do a normal render thereafter, it should only draw component 1
        menu.on_mouse_click(click1, &mut buddy);
        let result = menu.render(render_region, &mut buddy, false).unwrap();
        // This time, it should only have drawn the first component
        {
            let region = &result.drawn_region;
            assert!(region.is_inside(Point::new(0.0, 0.0)));
            assert!(region.is_inside(Point::new(0.5, 0.5)));
            assert!(!region.is_inside(Point::new(0.6, 0.6)));
        }
    }

    #[test]
    fn test_click_out() {
        struct ClickCountComponent {
            in_counter: Rc<Cell<u8>>,
            out_counter: Rc<Cell<u8>>,
        }

        impl Component for ClickCountComponent {
            fn on_attach(&mut self, buddy: &mut dyn ComponentBuddy) {
                buddy.subscribe_mouse_click();
                buddy.subscribe_mouse_click_out();
            }

            fn render(
                &mut self,
                _region: RenderRegion,
                _buddy: &mut dyn ComponentBuddy,
                _force: bool,
            ) -> RenderResult {
                Ok(RenderResultStruct {
                    drawn_region: Box::new(RectangularDrawnRegion::new(0.0, 0.0, 0.5, 0.5)),
                    filter_mouse_actions: true,
                })
            }

            fn on_mouse_click(&mut self, _event: MouseClickEvent, _buddy: &mut dyn ComponentBuddy) {
                self.in_counter.set(self.in_counter.get() + 1);
            }

            fn on_mouse_click_out(
                &mut self,
                _event: MouseClickOutEvent,
                _buddy: &mut dyn ComponentBuddy,
            ) {
                self.out_counter.set(self.out_counter.get() + 1);
            }
        }

        let mut buddy = root_buddy();
        let mut menu = SimpleFlatMenu::new(None);
        let in1 = Rc::new(Cell::new(0));
        let in2 = Rc::new(Cell::new(0));
        let out1 = Rc::new(Cell::new(0));
        let out2 = Rc::new(Cell::new(0));

        menu.add_component(
            Box::new(ClickCountComponent {
                in_counter: Rc::clone(&in1),
                out_counter: Rc::clone(&out1),
            }),
            ComponentDomain::between(0.0, 0.0, 0.5, 0.5),
        );
        menu.add_component(
            Box::new(ClickCountComponent {
                in_counter: Rc::clone(&in2),
                out_counter: Rc::clone(&out2),
            }),
            ComponentDomain::between(0.5, 0.5, 1.0, 1.0),
        );

        let click_miss =
            MouseClickEvent::new(Mouse::new(0), Point::new(0.8, 0.2), MouseButton::primary());
        let click_out = MouseClickOutEvent::new(Mouse::new(0), MouseButton::primary());
        let click1 =
            MouseClickEvent::new(Mouse::new(0), Point::new(0.2, 0.2), MouseButton::primary());
        let click2 =
            MouseClickEvent::new(Mouse::new(0), Point::new(0.6, 0.6), MouseButton::primary());
        let click1out =
            MouseClickEvent::new(Mouse::new(0), Point::new(0.3, 0.3), MouseButton::primary());
        let click2out =
            MouseClickEvent::new(Mouse::new(0), Point::new(0.9, 0.9), MouseButton::primary());

        let check_counters = |value_in1: u8, value_out1: u8, value_in2: u8, value_out2: u8| {
            assert_eq!(in1.get(), value_in1);
            assert_eq!(out1.get(), value_out1);
            assert_eq!(in2.get(), value_in2);
            assert_eq!(out2.get(), value_out2);
        };

        // Clicking before rendering shouldn't have any effect
        menu.on_mouse_click(click1, &mut buddy);
        check_counters(0, 0, 0, 0);

        // So let's render
        menu.render(RenderRegion::between(0, 0, 120, 10), &mut buddy, false)
            .unwrap();

        // Clicking outside the menu should increment both out counters
        menu.on_mouse_click_out(click_out, &mut buddy);
        check_counters(0, 1, 0, 1);

        // Clicking outside the components should also increment both out counters
        menu.on_mouse_click(click_miss, &mut buddy);
        check_counters(0, 2, 0, 2);

        // Clicking inside the render region of component 1...
        menu.on_mouse_click(click1, &mut buddy);
        check_counters(1, 2, 0, 3);

        // Clicking inside the render region of component 2...
        menu.on_mouse_click(click2, &mut buddy);
        check_counters(1, 3, 1, 3);

        // Clicking outside the render region of component 1
        // should increment both out counters
        menu.on_mouse_click(click1out, &mut buddy);
        check_counters(1, 4, 1, 4);

        // Same when clicking outside the render region of component 2
        menu.on_mouse_click(click2out, &mut buddy);
        check_counters(1, 5, 1, 5);
    }

    #[test]
    fn test_subscriptions() {
        struct SubscribingComponent {
            should_subscribe: Rc<Cell<bool>>,
            should_unsubscribe: Rc<Cell<bool>>,
            click_counter: Rc<Cell<u8>>,
            click_out_counter: Rc<Cell<u8>>,
        }

        impl Component for SubscribingComponent {
            fn on_attach(&mut self, _buddy: &mut dyn ComponentBuddy) {}

            fn render(
                &mut self,
                _region: RenderRegion,
                buddy: &mut dyn ComponentBuddy,
                _force: bool,
            ) -> RenderResult {
                if self.should_subscribe.get() {
                    buddy.subscribe_mouse_click();
                    buddy.subscribe_mouse_click_out();
                }
                if self.should_unsubscribe.get() {
                    buddy.unsubscribe_mouse_click();
                    buddy.unsubscribe_mouse_click_out();
                }
                self.should_subscribe.set(false);
                self.should_unsubscribe.set(false);
                entire_render_result()
            }

            fn on_mouse_click(&mut self, _event: MouseClickEvent, _buddy: &mut dyn ComponentBuddy) {
                self.click_counter.set(self.click_counter.get() + 1);
            }

            fn on_mouse_click_out(
                &mut self,
                _event: MouseClickOutEvent,
                _buddy: &mut dyn ComponentBuddy,
            ) {
                self.click_out_counter.set(self.click_out_counter.get() + 1);
            }
        }

        let subscribe1 = Rc::new(Cell::new(false));
        let subscribe2 = Rc::new(Cell::new(false));
        let unsubscribe1 = Rc::new(Cell::new(false));
        let unsubscribe2 = Rc::new(Cell::new(false));

        let click_count1 = Rc::new(Cell::new(0));
        let click_count2 = Rc::new(Cell::new(0));
        let click_out_count1 = Rc::new(Cell::new(0));
        let click_out_count2 = Rc::new(Cell::new(0));

        let buddy = root_buddy();
        let region = RenderRegion::between(0, 10, 20, 30);
        let mut menu = SimpleFlatMenu::new(None);
        menu.add_component(
            Box::new(SubscribingComponent {
                should_subscribe: Rc::clone(&subscribe1),
                should_unsubscribe: Rc::clone(&unsubscribe1),
                click_counter: Rc::clone(&click_count1),
                click_out_counter: Rc::clone(&click_out_count1),
            }),
            ComponentDomain::between(0.0, 0.0, 0.5, 0.5),
        );
        menu.add_component(
            Box::new(SubscribingComponent {
                should_subscribe: Rc::clone(&subscribe2),
                should_unsubscribe: Rc::clone(&unsubscribe2),
                click_counter: Rc::clone(&click_count2),
                click_out_counter: Rc::clone(&click_out_count2),
            }),
            ComponentDomain::between(0.5, 0.5, 1.0, 1.0),
        );

        let menu_cell = Rc::new(RefCell::new(menu));
        let buddy_cell = Rc::new(RefCell::new(buddy));
        let do_render = || {
            let mut menu = menu_cell.borrow_mut();
            let mut buddy = buddy_cell.borrow_mut();
            menu.render(region, &mut *buddy, true).unwrap();
        };

        let fire_click = || {
            let mut menu = menu_cell.borrow_mut();
            let mut buddy = buddy_cell.borrow_mut();
            menu.on_mouse_click(
                MouseClickEvent::new(Mouse::new(0), Point::new(0.2, 0.2), MouseButton::primary()),
                &mut *buddy,
            );
            menu.on_mouse_click(
                MouseClickEvent::new(Mouse::new(0), Point::new(0.8, 0.8), MouseButton::primary()),
                &mut *buddy,
            );
        };
        let fire_click_out = || {
            let mut menu = menu_cell.borrow_mut();
            let mut buddy = buddy_cell.borrow_mut();
            menu.on_mouse_click_out(
                MouseClickOutEvent::new(Mouse::new(0), MouseButton::primary()),
                &mut *buddy,
            );
        };

        let check_values = |click1: u8, click_out1: u8, click2: u8, click_out2: u8| {
            assert_eq!(click1, click_count1.get());
            assert_eq!(click_out1, click_out_count1.get());
            assert_eq!(click2, click_count2.get());
            assert_eq!(click_out2, click_out_count2.get());
        };

        // No subscriptions yet, so these events should be ignored
        do_render();
        fire_click();
        fire_click_out();
        check_values(0, 0, 0, 0);

        // Lets subscribe the first component
        subscribe1.set(true);
        do_render();
        fire_click();
        check_values(1, 1, 0, 0);

        // Now the second one as well
        subscribe2.set(true);
        do_render();
        fire_click_out();
        check_values(1, 2, 0, 1);

        // Nah, let's cancel the subscription for the second one
        unsubscribe2.set(true);
        do_render();
        fire_click();
        check_values(2, 3, 0, 1);

        // This is not fair... lets cancel the first one as well
        unsubscribe1.set(true);
        do_render();
        fire_click_out();
        check_values(2, 3, 0, 1);

        // Lets give the second one a comeback
        subscribe2.set(true);
        do_render();
        fire_click();
        fire_click_out();
        check_values(2, 3, 1, 3);

        // Let's stop
        unsubscribe2.set(true);
        do_render();
        fire_click();
        fire_click_out();
        check_values(2, 3, 1, 3);
    }

    struct MouseMotionComponent {

        should_filter_mouse_actions: Rc<Cell<bool>>,
        mouse_move_log: Rc<RefCell<Vec<MouseMoveEvent>>>,
        mouse_enter_log: Rc<RefCell<Vec<MouseEnterEvent>>>,
        mouse_leave_log: Rc<RefCell<Vec<MouseLeaveEvent>>>,
    }

    impl Component for MouseMotionComponent {
        fn on_attach(&mut self, buddy: &mut dyn ComponentBuddy) {
            buddy.subscribe_mouse_move();
            buddy.subscribe_mouse_enter();
            buddy.subscribe_mouse_leave();
        }

        fn render(&mut self, _region: RenderRegion, _buddy: &mut dyn ComponentBuddy, _force: bool) -> RenderResult {
            Ok(RenderResultStruct {
                filter_mouse_actions: self.should_filter_mouse_actions.get(),
                drawn_region: Box::new(RectangularDrawnRegion::new(0.2, 0.2, 0.8, 0.8))
            })
        }

        fn on_mouse_move(&mut self, event: MouseMoveEvent, _buddy: &mut dyn ComponentBuddy) {
            let mut move_log = self.mouse_move_log.borrow_mut();
            move_log.push(event);
        }

        fn on_mouse_enter(&mut self, event: MouseEnterEvent, _buddy: &mut dyn ComponentBuddy) {
            let mut enter_log = self.mouse_enter_log.borrow_mut();
            enter_log.push(event);
        }

        fn on_mouse_leave(&mut self, event: MouseLeaveEvent, _buddy: &mut dyn ComponentBuddy) {
            let mut leave_log = self.mouse_leave_log.borrow_mut();
            leave_log.push(event);
        }
    }

    #[test]
    fn test_mouse_enter_and_leave() {
        let enter_log1 = Rc::new(RefCell::new(Vec::new()));
        let leave_log1 = Rc::new(RefCell::new(Vec::new()));
        let enter_log2 = Rc::new(RefCell::new(Vec::new()));
        let leave_log2 = Rc::new(RefCell::new(Vec::new()));

        let component1 = MouseMotionComponent {
            should_filter_mouse_actions: Rc::new(Cell::new(true)),
            mouse_move_log: Rc::new(RefCell::new(Vec::new())),
            mouse_enter_log: Rc::clone(&enter_log1),
            mouse_leave_log: Rc::clone(&leave_log1)
        };
        let component2 = MouseMotionComponent {
            should_filter_mouse_actions: Rc::new(Cell::new(true)),
            mouse_move_log: Rc::new(RefCell::new(Vec::new())),
            mouse_enter_log: Rc::clone(&enter_log2),
            mouse_leave_log: Rc::clone(&leave_log2)
        };

        let mut buddy = root_buddy();
        let mut menu = SimpleFlatMenu::new(None);
        menu.on_attach(&mut buddy);
        menu.add_component(
            Box::new(component1),
            ComponentDomain::between(0.1, 0.1, 0.4, 0.9)
        );
        menu.add_component(
            Box::new(component2),
            ComponentDomain::between(0.6, 0.1, 0.9, 0.9)
        );

        let miss_enter_event = MouseEnterEvent::new(
            Mouse::new(0), Point::new(0.5, 0.5)
        );
        let miss_leave_event = MouseLeaveEvent::new(
            Mouse::new(0), Point::new(0.5, 0.5)
        );
        let edge_enter_event = MouseEnterEvent::new(
            Mouse::new(0), Point::new(0.65, 0.5)
        );
        let edge_leave_event = MouseLeaveEvent::new(
            Mouse::new(0), Point::new(0.65, 0.5)
        );
        let hit_enter_event = MouseEnterEvent::new(
            Mouse::new(0), Point::new(0.75, 0.5)
        );
        let hit_leave_event = MouseLeaveEvent::new(
            Mouse::new(0), Point::new(0.75, 0.5)
        );
        let render_region = RenderRegion::between(1, 2, 3, 4);

        // Nothing should happen before the first render
        menu.on_mouse_enter(hit_enter_event, &mut buddy);
        menu.on_mouse_leave(hit_leave_event, &mut buddy);

        // So let's render
        menu.render(render_region, &mut buddy, false).unwrap();

        // Due to the mouse filtering, the edge event shouldn't trigger any reaction either
        menu.on_mouse_enter(edge_enter_event, &mut buddy);
        menu.on_mouse_leave(edge_leave_event, &mut buddy);

        // But, the hit events should have effect
        menu.on_mouse_enter(hit_enter_event, &mut buddy);
        menu.on_mouse_leave(hit_leave_event, &mut buddy);

        // The miss events shouldn't have any effect anyway
        menu.on_mouse_enter(miss_enter_event, &mut buddy);
        menu.on_mouse_leave(miss_leave_event, &mut buddy);

        let enter_log1 = enter_log1.borrow();
        assert!(enter_log1.is_empty());
        let leave_log1 = leave_log1.borrow();
        assert!(leave_log1.is_empty());

        let enter_log2 = enter_log2.borrow();
        assert_eq!(1, enter_log2.len());
        assert!(enter_log2[0].get_entrance_point().nearly_equal(Point::new(0.5, 0.5)));

        let leave_log2 = leave_log2.borrow();
        assert_eq!(1, leave_log2.len());
        assert!(leave_log2[0].get_exit_point().nearly_equal(Point::new(0.5, 0.5)));
    }

    #[test]
    fn test_mouse_move() {
        let move_logs = vec![
            Rc::new(RefCell::new(Vec::new())),
            Rc::new(RefCell::new(Vec::new())),
            Rc::new(RefCell::new(Vec::new())),
            Rc::new(RefCell::new(Vec::new())),
            Rc::new(RefCell::new(Vec::new())),
        ];
        let enter_logs = vec![
            Rc::new(RefCell::new(Vec::new())),
            Rc::new(RefCell::new(Vec::new())),
            Rc::new(RefCell::new(Vec::new())),
            Rc::new(RefCell::new(Vec::new())),
            Rc::new(RefCell::new(Vec::new())),
        ];
        let leave_logs = vec![
            Rc::new(RefCell::new(Vec::new())),
            Rc::new(RefCell::new(Vec::new())),
            Rc::new(RefCell::new(Vec::new())),
            Rc::new(RefCell::new(Vec::new())),
            Rc::new(RefCell::new(Vec::new())),
        ];

        let mut menu = SimpleFlatMenu::new(None);
        let mut buddy = root_buddy();
        menu.on_attach(&mut buddy);

        // The outer bottom-left component
        menu.add_component(Box::new(MouseMotionComponent {
            should_filter_mouse_actions: Rc::new(Cell::new(true)),
            mouse_move_log: Rc::clone(&move_logs[0]),
            mouse_enter_log: Rc::clone(&enter_logs[0]),
            mouse_leave_log: Rc::clone(&leave_logs[0]),
        }), ComponentDomain::between(0.0, 0.0, 0.25, 0.25));

        // The inner bottom-left component
        menu.add_component(Box::new(MouseMotionComponent {
            should_filter_mouse_actions: Rc::new(Cell::new(false)),
            mouse_move_log: Rc::clone(&move_logs[1]),
            mouse_enter_log: Rc::clone(&enter_logs[1]),
            mouse_leave_log: Rc::clone(&leave_logs[1]),
        }), ComponentDomain::between(0.25, 0.25, 0.5, 0.5));

        // The inner top-right component
        menu.add_component(Box::new(MouseMotionComponent {
            should_filter_mouse_actions: Rc::new(Cell::new(true)),
            mouse_move_log: Rc::clone(&move_logs[2]),
            mouse_enter_log: Rc::clone(&enter_logs[2]),
            mouse_leave_log: Rc::clone(&leave_logs[2]),
        }), ComponentDomain::between(0.5, 0.5, 0.75, 0.75));

        // The outer top-right component
        menu.add_component(Box::new(MouseMotionComponent {
            should_filter_mouse_actions: Rc::new(Cell::new(true)),
            mouse_move_log: Rc::clone(&move_logs[4]),
            mouse_enter_log: Rc::clone(&enter_logs[4]),
            mouse_leave_log: Rc::clone(&leave_logs[4]),
        }), ComponentDomain::between(0.75, 0.75, 1.0, 1.0));

        // This component should be missed entirely
        menu.add_component(Box::new(MouseMotionComponent {
            should_filter_mouse_actions: Rc::new(Cell::new(false)),
            mouse_move_log: Rc::clone(&move_logs[3]),
            mouse_enter_log: Rc::clone(&enter_logs[3]),
            mouse_leave_log: Rc::clone(&leave_logs[3]),
        }), ComponentDomain::between(0.5, 0.0, 0.75, 0.25));

        menu.render(
            RenderRegion::between(0, 0, 20, 30),
            &mut buddy, false
        ).unwrap();

        let mouse = Mouse::new(3);
        let entrance_x = 0.25 * 0.25;
        let entrance_y = 0.25 * 0.25;
        let entrance = Point::new(entrance_x, entrance_y);
        let exit_x = 1.0 - entrance_x;
        let exit_y = 1.0 - entrance_y;
        let exit = Point::new(exit_x, exit_y);

        let enter_event = MouseEnterEvent::new(mouse, entrance);
        let move_event = MouseMoveEvent::new(
            mouse, entrance, exit
        );
        let leave_event = MouseLeaveEvent::new(mouse, exit);
        menu.on_mouse_enter(enter_event, &mut buddy);
        menu.on_mouse_move(move_event, &mut buddy);
        menu.on_mouse_leave(leave_event, &mut buddy);

        // Time to check the results...

        // But first some helper functions
        let eq_mouse_move = |enter_x: f32, enter_y: f32, exit_x: f32, exit_y: f32, event: &MouseMoveEvent| {
            assert_eq!(mouse, event.get_mouse());
            assert!(Point::new(enter_x, enter_y).nearly_equal(event.get_from()));
            assert!(Point::new(exit_x, exit_y).nearly_equal(event.get_to()));
        };
        let eq_mouse_enter = |enter_x: f32, enter_y: f32, event: &MouseEnterEvent| {
            assert_eq!(mouse, event.get_mouse());
            assert!(Point::new(enter_x, enter_y).nearly_equal(event.get_entrance_point()));
        };
        let eq_mouse_leave = |exit_x: f32, exit_y: f32, event: &MouseLeaveEvent| {
            assert_eq!(mouse, event.get_mouse());
            assert!(Point::new(exit_x, exit_y).nearly_equal(event.get_exit_point()));
        };
        let check_log = |index: usize, enter_x: f32, enter_y: f32, exit_x: f32, exit_y: f32| {
            let move_log = move_logs[index].borrow();
            assert_eq!(1, move_log.len());
            eq_mouse_move(enter_x, enter_y, exit_x, exit_y, &move_log[0]);
            let enter_log = enter_logs[index].borrow();
            assert_eq!(1, enter_log.len());
            eq_mouse_enter(enter_x, enter_y, &enter_log[0]);
            let leave_log = leave_logs[index].borrow();
            assert_eq!(1, leave_log.len());
            eq_mouse_leave(exit_x, exit_y, &leave_log[0]);
        };

        // Finally check the actual results
        check_log(0, 0.25, 0.25, 0.8, 0.8);
        check_log(1, 0.0, 0.0, 1.0, 1.0);
        check_log(2, 0.2, 0.2, 0.8, 0.8);
        check_log(4, 0.2, 0.2, 0.75, 0.75);

        // And check that the out-of-line component didn't receive any events
        let move_log = move_logs[3].borrow();
        assert!(move_log.is_empty());
        let enter_log = enter_logs[3].borrow();
        assert!(enter_log.is_empty());
        let leave_log = leave_logs[3].borrow();
        assert!(leave_log.is_empty());
    }

    // TODO Test mouse move, enter, and leave subscriptions
    // TODO Test mouse move fully inside

    #[test]
    fn test_mouse_move_subscriptions() {
        struct MouseMoveSubscribeComponent {
            subscribe_mouse_move: Rc<Cell<bool>>,
            subscribe_mouse_enter: Rc<Cell<bool>>,
            subscribe_mouse_leave: Rc<Cell<bool>>,

            mouse_move_log: Rc<RefCell<Vec<MouseMoveEvent>>>,
            mouse_enter_log: Rc<RefCell<Vec<MouseEnterEvent>>>,
            mouse_leave_log: Rc<RefCell<Vec<MouseLeaveEvent>>>,
        }

        impl Component for MouseMoveSubscribeComponent {
            fn on_attach(&mut self, _buddy: &mut dyn ComponentBuddy) {}

            fn render(
                &mut self, _region: RenderRegion, buddy: &mut dyn ComponentBuddy, _force: bool
            ) -> RenderResult {
                if self.subscribe_mouse_move.get() {
                    buddy.subscribe_mouse_move();
                } else {
                    buddy.unsubscribe_mouse_move();
                }
                if self.subscribe_mouse_enter.get() {
                    buddy.subscribe_mouse_enter();
                } else {
                    buddy.unsubscribe_mouse_enter();
                }
                if self.subscribe_mouse_leave.get() {
                    buddy.subscribe_mouse_leave();
                } else {
                    buddy.unsubscribe_mouse_leave();
                }
                entire_render_result()
            }

            fn on_mouse_move(&mut self, event: MouseMoveEvent, _buddy: &mut dyn ComponentBuddy) {
                let mut move_log = self.mouse_move_log.borrow_mut();
                move_log.push(event);
            }

            fn on_mouse_enter(&mut self, event: MouseEnterEvent, _buddy: &mut dyn ComponentBuddy) {
                let mut enter_log = self.mouse_enter_log.borrow_mut();
                enter_log.push(event);
            }

            fn on_mouse_leave(&mut self, event: MouseLeaveEvent, _buddy: &mut dyn ComponentBuddy) {
                let mut leave_log = self.mouse_leave_log.borrow_mut();
                leave_log.push(event);
            }
        }

        let mouse_move_log = Rc::new(RefCell::new(Vec::new()));
        let mouse_enter_log = Rc::new(RefCell::new(Vec::new()));
        let mouse_leave_log = Rc::new(RefCell::new(Vec::new()));

        let sub_mouse_move = Rc::new(Cell::new(false));
        let sub_mouse_enter = Rc::new(Cell::new(false));
        let sub_mouse_leave = Rc::new(Cell::new(false));

        let component = MouseMoveSubscribeComponent {
            mouse_move_log: Rc::clone(&mouse_move_log),
            mouse_enter_log: Rc::clone(&mouse_enter_log),
            mouse_leave_log: Rc::clone(&mouse_leave_log),

            subscribe_mouse_move: Rc::clone(&sub_mouse_move),
            subscribe_mouse_enter: Rc::clone(&sub_mouse_enter),
            subscribe_mouse_leave: Rc::clone(&sub_mouse_leave),
        };

        let mut menu = SimpleFlatMenu::new(None);
        let mut buddy = root_buddy();

        menu.on_attach(&mut buddy);
        menu.add_component(
            Box::new(component),
            ComponentDomain::between(0.0, 0.0, 0.5, 0.5)
        );

        let mut try_combination = |mouse_move: bool, mouse_enter: bool, mouse_leave: bool| {
            sub_mouse_move.set(mouse_move);
            sub_mouse_enter.set(mouse_enter);
            sub_mouse_leave.set(mouse_leave);
            menu.render(
                RenderRegion::between(0, 1, 4, 7),
                &mut buddy, true
            ).unwrap();
            let mouse = Mouse::new(2);
            let original_enter_event1 = MouseEnterEvent::new(
                mouse, Point::new(0.1, 0.6)
            );
            let original_enter_event2 = MouseMoveEvent::new(
                mouse, Point::new(0.1, 0.6), Point::new(0.1, 0.25)
            );
            let original_move_event = MouseMoveEvent::new(
                mouse, Point::new(0.1, 0.25), Point::new(0.4, 0.25)
            );
            let original_leave_event1 = MouseMoveEvent::new(
                mouse, Point::new(0.4, 0.25), Point::new(0.4, 0.6)
            );
            let original_leave_event2 = MouseLeaveEvent::new(
                mouse, Point::new(0.4, 0.6)
            );
            let transformed_enter_event1 = MouseEnterEvent::new(
                mouse, Point::new(0.2, 1.0)
            );
            let transformed_enter_event2 = MouseMoveEvent::new(
                mouse, Point::new(0.2, 1.0), Point::new(0.2, 0.5)
            );
            let transformed_move_event = MouseMoveEvent::new(
                mouse, Point::new(0.2, 0.5), Point::new(0.8, 0.5)
            );
            let transformed_leave_event1 = MouseMoveEvent::new(
                mouse, Point::new(0.8, 0.5), Point::new(0.8, 1.0)
            );
            let transformed_leave_event2 = MouseLeaveEvent::new(
                mouse, Point::new(0.8, 1.0)
            );

            menu.on_mouse_enter(original_enter_event1, &mut buddy);
            menu.on_mouse_move(original_enter_event2, &mut buddy);
            menu.on_mouse_move(original_move_event, &mut buddy);
            menu.on_mouse_move(original_leave_event1, &mut buddy);
            menu.on_mouse_leave(original_leave_event2, &mut buddy);

            let mut move_log = mouse_move_log.borrow_mut();
            let mut enter_log = mouse_enter_log.borrow_mut();
            let mut leave_log = mouse_leave_log.borrow_mut();

            if mouse_move {
                let move_event_eq = |expected: &MouseMoveEvent, actual: &MouseMoveEvent| {
                    assert_eq!(expected.get_mouse(), actual.get_mouse());
                    assert!(expected.get_from().nearly_equal(actual.get_from()));
                    assert!(expected.get_to().nearly_equal(actual.get_to()));
                };

                assert_eq!(3, move_log.len());
                move_event_eq(&transformed_enter_event2, &move_log[0]);
                move_event_eq(&transformed_move_event, &move_log[1]);
                move_event_eq(&transformed_leave_event1, &move_log[2]);
            } else {
                assert!(move_log.is_empty());
            }

            if mouse_enter {
                let enter_event_eq = |expected: &MouseEnterEvent, actual: &MouseEnterEvent| {
                    assert_eq!(expected.get_mouse(), actual.get_mouse());
                    assert!(expected.get_entrance_point().nearly_equal(actual.get_entrance_point()));
                };

                assert_eq!(1, enter_log.len());
                enter_event_eq(&transformed_enter_event1, &enter_log[0]);
            } else {
                assert!(enter_log.is_empty());
            }

            if mouse_leave {
                let leave_event_eq = |expected: &MouseLeaveEvent, actual: &MouseLeaveEvent| {
                    assert_eq!(expected.get_mouse(), actual.get_mouse());
                    assert!(expected.get_exit_point().nearly_equal(actual.get_exit_point()));
                };

                assert_eq!(1, leave_log.len());
                leave_event_eq(&transformed_leave_event2, &leave_log[0]);
            } else {
                assert!(leave_log.is_empty());
            }

            move_log.clear();
            enter_log.clear();
            leave_log.clear();
        };

        for _counter in 0 .. 2 {
            try_combination(false, false, false);
            try_combination(false, false, true);
            try_combination(false, true, false);
            try_combination(false, true, true);
            try_combination(true, false, false);
            try_combination(true, false, true);
            try_combination(true, true, false);
            try_combination(true, true, true);
        }
    }

    #[test]
    fn test_own_subscriptions() {
        struct CuriousComponent {}

        impl Component for CuriousComponent {
            fn on_attach(&mut self, buddy: &mut dyn ComponentBuddy) {
                buddy.subscribe_mouse_click();
                buddy.subscribe_mouse_click_out();
                buddy.subscribe_mouse_move();
                buddy.subscribe_mouse_enter();
                buddy.subscribe_mouse_leave();
            }

            fn render(&mut self, _region: RenderRegion, _buddy: &mut dyn ComponentBuddy, _force: bool) -> RenderResult {
                entire_render_result()
            }
        }

        let mut menu = SimpleFlatMenu::new(None);
        let mut buddy = root_buddy();
        menu.on_attach(&mut buddy);
        menu.add_component(
            Box::new(CuriousComponent {}),
            ComponentDomain::between(0.3, 0.6, 1.0, 0.9)
        );

        // The menu should have subscribed to all events
        let subs = buddy.get_subscriptions();
        assert!(subs.mouse_click);
        assert!(subs.mouse_click_out);
        assert!(subs.mouse_move);
        assert!(subs.mouse_enter);
        assert!(subs.mouse_leave);
    }
    
    #[test]
    fn test_buddy_get_all_mouses() {
        struct GetMouseComponent {
            expected: Rc<RefCell<Vec<Mouse>>>,
            call_counter: Rc<Cell<u8>>,
        }

        impl Component for GetMouseComponent {
            fn on_attach(&mut self, _buddy: &mut dyn ComponentBuddy) {}

            fn render(&mut self, _region: RenderRegion, buddy: &mut dyn ComponentBuddy, _force: bool) -> RenderResult {
                let expected = self.expected.borrow();
                assert_eq!(expected.as_ref() as &Vec<Mouse>, &buddy.get_all_mouses());
                self.call_counter.set(self.call_counter.get() + 1);
                entire_render_result()
            }
        }

        let expected_mouses = Rc::new(RefCell::new(Vec::new()));
        let call_counter = Rc::new(Cell::new(0));

        let mut menu = SimpleFlatMenu::new(None);
        menu.add_component(
            Box::new(GetMouseComponent {
                expected: Rc::clone(&expected_mouses),
                call_counter: Rc::clone(&call_counter)
            }),
            ComponentDomain::between(0.1, 0.2, 0.3, 0.4)
        );

        let mut application = Application::new(
            Box::new(menu)
        );

        let region = RenderRegion::with_size(1, 2, 3, 4);

        // The mouses should be empty initially
        application.render(region, true);

        let enter_event = |mouse_id: u16| MouseEnterEvent::new(
            Mouse::new(mouse_id), Point::new(0.2, 0.3)
        );
        let leave_event = |mouse_id: u16| MouseLeaveEvent::new(
            Mouse::new(mouse_id), Point::new(0.2, 0.3)
        );
        let mouse_vec = |ids: &[u16]| ids.iter().map(|id| Mouse::new(*id)).collect();

        // Add the first mouse
        application.fire_mouse_enter_event(enter_event(123));
        expected_mouses.replace(mouse_vec(&[123]));
        application.render(region, true);

        // Add the second mouse
        application.fire_mouse_enter_event(enter_event(1));
        expected_mouses.replace(mouse_vec(&[123, 1]));
        application.render(region, true);

        // Remove the first mouse
        application.fire_mouse_leave_event(leave_event(123));
        expected_mouses.replace(mouse_vec(&[1]));
        application.render(region, true);

        // Add the first mouse back, and add yet another mouse
        application.fire_mouse_enter_event(enter_event(123));
        application.fire_mouse_enter_event(enter_event(8));
        expected_mouses.replace(mouse_vec(&[1, 123, 8]));
        application.render(region, true);

        // Remove all mouses
        application.fire_mouse_leave_event(leave_event(123));
        application.fire_mouse_leave_event(leave_event(8));
        application.fire_mouse_leave_event(leave_event(1));
        expected_mouses.replace(mouse_vec(&[]));
        application.render(region, true);

        assert_eq!(6, call_counter.get());
    }

    #[test]
    fn test_buddy_get_local_mouses_and_positions() {
        struct LocalMouse {
            mouse: Mouse,
            position: Point
        }

        struct LocalMouseCheckComponent {
            expected_mouses: Rc<RefCell<Vec<LocalMouse>>>
        }

        impl Component for LocalMouseCheckComponent {
            fn on_attach(&mut self, buddy: &mut dyn ComponentBuddy) {}

            fn render(&mut self, region: RenderRegion, buddy: &mut dyn ComponentBuddy, force: bool) -> RenderResult {
                let local_mouses = buddy.get_local_mouses();
                let expected_mouses = self.expected_mouses.borrow();
                assert_eq!(expected_mouses.len(), local_mouses.len());

                // Test that all local mouses are present and have the right position
                'outer: for mouse in &local_mouses {
                    for entry in &*expected_mouses {
                        if entry.mouse == *mouse {
                            assert!(entry.position.nearly_equal(buddy.get_mouse_position(*mouse).unwrap()));
                            continue 'outer;
                        }
                    }
                    panic!("Expected mouse {:?}, but didn't find its entry", mouse);
                }

                // Test that the other mouses do not have a position
                'outer: for mouse in buddy.get_all_mouses() {
                    for local_mouse in &local_mouses {
                        if *local_mouse == mouse {
                            continue 'outer;
                        }
                    }
                    assert!(buddy.get_mouse_position(mouse).is_none());
                }

                // Neither should a non-existing mouse
                assert!(buddy.get_mouse_position(Mouse::new(8347)).is_none());

                entire_render_result()
            }
        }

        let expected_mouses1 = Rc::new(RefCell::new(Vec::new()));
        let expected_mouses2 = Rc::new(RefCell::new(Vec::new()));

        let set = |target: &Rc<RefCell<Vec<LocalMouse>>>, mut expected: Vec<LocalMouse>| {
            let mut mouses = target.borrow_mut();
            mouses.clear();
            mouses.append(&mut expected);
        };
        let set1 = |mut expected: Vec<LocalMouse>| set(&expected_mouses1, expected);
        let set2 = |mut expected: Vec<LocalMouse>| set(&expected_mouses2, expected);

        let mut menu = SimpleFlatMenu::new(None);
        menu.add_component(
            Box::new(LocalMouseCheckComponent {
                expected_mouses: Rc::clone(&expected_mouses1)
            }), ComponentDomain::between(0.2, 0.0, 0.5, 0.7)
        );
        menu.add_component(
            Box::new(LocalMouseCheckComponent {
                expected_mouses: Rc::clone(&expected_mouses2)
            }), ComponentDomain::between(0.5, 0.5, 1.0, 1.0)
        );

        let mut application = Application::new(Box::new(menu));
        let region = RenderRegion::between(10, 20, 30, 40);
        application.render(region, true);

        // Start with 1 mouse, and spawn it in the middle of the first component
        let mouse1 = Mouse::new(6);
        application.fire_mouse_enter_event(MouseEnterEvent::new(
            mouse1, Point::new(0.35, 0.35)
        ));
        set1(vec![LocalMouse { mouse: mouse1, position: Point::new(0.5, 0.5)}]);
        set2(vec![]);
        application.render(region, true);

        // Move the mouse to the other component
        application.fire_mouse_move_event(MouseMoveEvent::new(
            mouse1, Point::new(0.35, 0.35), Point::new(0.6, 0.9)
        ));
        set1(vec![]);
        set2(vec![LocalMouse { mouse: mouse1, position: Point::new(0.2, 0.8) }]);
        application.render(region, true);

        // Move the mouse away from both components
        application.fire_mouse_move_event(MouseMoveEvent::new(
            mouse1, Point::new(0.6, 0.9), Point::new(0.1, 0.1)
        ));
        set1(vec![]);
        set2(vec![]);
        application.render(region, true);

        // Introduce the second mouse
        let mouse2 = Mouse::new(120);
        application.fire_mouse_enter_event(MouseEnterEvent::new(
            mouse2, Point::new(0.1, 0.1)
        ));
        // Neither of the mouses is inside any of the components
        application.render(region, true);

        // Move the second mouse to the second component
        application.fire_mouse_move_event(MouseMoveEvent::new(
            mouse2, Point::new(0.1, 0.1), Point::new(0.7, 0.8)
        ));
        set1(vec![]);
        set2(vec![LocalMouse { mouse: mouse2, position: Point::new(0.4, 0.6) }]);
        application.render(region, true);

        // Move the first mouse to the first component
        application.fire_mouse_move_event(MouseMoveEvent::new(
            mouse1, Point::new(0.1, 0.1), Point::new(0.35, 0.35)
        ));
        set1(vec![LocalMouse { mouse: mouse1, position: Point::new(0.5, 0.5) }]);
        set2(vec![LocalMouse { mouse: mouse2, position: Point::new(0.4, 0.6) }]);
        application.render(region, true);

        // Move the first mouse to the second component
        application.fire_mouse_move_event(MouseMoveEvent::new(
            mouse1, Point::new(0.35, 0.35), Point::new(0.8, 0.7)
        ));
        set1(vec![]);
        set2(vec![
            LocalMouse { mouse: mouse1, position: Point::new(0.6, 0.4) },
            LocalMouse { mouse: mouse2, position: Point::new(0.4, 0.6) },
        ]);
        application.render(region, true);

        // Remove the second mouse
        application.fire_mouse_leave_event(MouseLeaveEvent::new(
            mouse2, Point::new(0.7, 0.8)
        ));
        set1(vec![]);
        set2(vec![LocalMouse { mouse: mouse1, position: Point::new(0.6, 0.4) }]);
        application.render(region, true);
    }
}
