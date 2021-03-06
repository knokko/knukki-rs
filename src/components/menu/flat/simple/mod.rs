use crate::*;

use std::cell::RefCell;
use std::rc::Rc;

mod buddy;
mod domain;

use buddy::*;
pub use domain::*;

type RR<T> = Rc<RefCell<T>>;
//type WR<T> = Weak<RefCell<T>>;

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
            let should_have_pressed_buttons = own_buddy.get_pressed_mouse_buttons(mouse);
            if let Some(position) = should_have_position {
                if let Some(pressed_buttons) = should_have_pressed_buttons {
                    mouse_buddy.local_mouses.push(MouseEntry {
                        mouse,
                        position,
                        pressed_buttons,
                    });
                } else {
                    // This is weird behavior that should be investigated, but not worth a production
                    // crash
                    debug_assert!(false);
                }
            } else {
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

            if entry.buddy.has_next_menu() {
                own_buddy.change_menu(entry.buddy.create_next_menu());
            }

            entry.buddy.clear_changes();
        }
    }

    fn get_component_at(&self, point: Point) -> Option<RR<ComponentEntry>> {
        // TODO PERFORMANCE Use some kind of 2d range tree instead
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
        buddy.subscribe_mouse_press();
        buddy.subscribe_mouse_release();
        buddy.subscribe_mouse_move();
        buddy.subscribe_mouse_enter();
        buddy.subscribe_mouse_leave();
    }

    // Variables only used when the golem_rendering feature is enabled are
    // considered 'unused' when compiling without this feature.
    #[allow(unused_variables)]
    fn render(
        &mut self,
        renderer: &Renderer,
        buddy: &mut dyn ComponentBuddy,
        force: bool,
    ) -> RenderResult {
        // This needs to happen before each event
        self.update_internal(buddy, true);

        // Now onto the 'actual' drawing
        if force || !self.has_rendered_before {
            if let Some(background_color) = self.background_color {
                // TODO And take more care when this is partially transparent...
                renderer.clear(background_color);
            }
        }
        let mut drawn_regions: Vec<Box<dyn DrawnRegion>> = Vec::new();
        for entry_cell in &self.components {
            let mut entry = entry_cell.borrow_mut();
            let component_domain = entry.domain;

            if let Some(entry_result) = entry.render(renderer, force) {
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

        // TODO PERFORMANCE Maintain a list for just the interested components
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
        self.update_internal(own_buddy, false);

        // TODO PERFORMANCE Maintain a list for just the interested components
        for component_cell in &self.components {
            let mut component_entry = component_cell.borrow_mut();
            component_entry.mouse_click_out(event);
            self.check_buddy(own_buddy, &mut component_entry, false);
        }
    }

    fn on_mouse_press(&mut self, event: MousePressEvent, own_buddy: &mut dyn ComponentBuddy) {
        // This should be done before every important action
        self.update_internal(own_buddy, false);

        // Lets now handle the actual press event
        let maybe_clicked_cell = self.get_component_at(event.get_point());

        if let Some(clicked_cell) = &maybe_clicked_cell {
            let mut clicked_entry = clicked_cell.borrow_mut();
            clicked_entry.mouse_press(event);
            self.check_buddy(own_buddy, &mut clicked_entry, false);
        }
    }

    fn on_mouse_release(&mut self, event: MouseReleaseEvent, own_buddy: &mut dyn ComponentBuddy) {
        // This should be done before every important action
        self.update_internal(own_buddy, false);

        // Lets now handle the actual press event
        let maybe_clicked_cell = self.get_component_at(event.get_point());

        if let Some(clicked_cell) = &maybe_clicked_cell {
            let mut clicked_entry = clicked_cell.borrow_mut();
            clicked_entry.mouse_release(event);
            self.check_buddy(own_buddy, &mut clicked_entry, false);
        }
    }

    fn on_mouse_move(&mut self, event: MouseMoveEvent, own_buddy: &mut dyn ComponentBuddy) {
        self.update_internal(own_buddy, false);

        // TODO PERFORMANCE Consider only the components intersecting the rectangle around the line from
        // event.from to event.to (using some kind of 2d range tree)
        for entry_cell in &self.components {
            let mut entry = entry_cell.borrow_mut();
            entry.mouse_move(event);
            self.check_buddy(own_buddy, &mut entry, false);
        }
    }

    fn on_mouse_enter(&mut self, event: MouseEnterEvent, own_buddy: &mut dyn ComponentBuddy) {
        self.update_internal(own_buddy, false);

        if let Some(hit_component_entry) = self.get_component_at(event.get_entrance_point()) {
            let mut borrowed_entry = hit_component_entry.borrow_mut();
            borrowed_entry.mouse_enter(event);
            self.check_buddy(own_buddy, &mut borrowed_entry, false);
        }
    }

    fn on_mouse_leave(&mut self, event: MouseLeaveEvent, own_buddy: &mut dyn ComponentBuddy) {
        self.update_internal(own_buddy, false);

        if let Some(hit_component_entry) = self.get_component_at(event.get_exit_point()) {
            let mut borrowed_entry = hit_component_entry.borrow_mut();
            borrowed_entry.mouse_leave(event);
            self.check_buddy(own_buddy, &mut borrowed_entry, false);
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

    fn mouse_press(&mut self, outer_event: MousePressEvent) {
        if self.buddy.get_subscriptions().mouse_press {
            let transformed_point = self.domain.transform(outer_event.get_point());
            if let Some(render_result) = self.buddy.get_last_render_result() {
                if !render_result.filter_mouse_actions
                    || render_result.drawn_region.is_inside(transformed_point)
                {
                    let transformed_event = MousePressEvent::new(
                        outer_event.get_mouse(),
                        transformed_point,
                        outer_event.get_button(),
                    );

                    self.component
                        .on_mouse_press(transformed_event, &mut self.buddy);
                }
            }
        }
    }

    fn mouse_release(&mut self, outer_event: MouseReleaseEvent) {
        if self.buddy.get_subscriptions().mouse_release {
            let transformed_point = self.domain.transform(outer_event.get_point());
            if let Some(render_result) = self.buddy.get_last_render_result() {
                if !render_result.filter_mouse_actions
                    || render_result.drawn_region.is_inside(transformed_point)
                {
                    let transformed_event = MouseReleaseEvent::new(
                        outer_event.get_mouse(),
                        transformed_point,
                        outer_event.get_button(),
                    );

                    self.component
                        .on_mouse_release(transformed_event, &mut self.buddy);
                }
            }
        }
    }

    fn mouse_enter(&mut self, event: MouseEnterEvent) {
        if self.buddy.get_subscriptions().mouse_enter {
            if let Some(render_result) = self.buddy.get_last_render_result() {
                let transformed_entrance_point = self.domain.transform(event.get_entrance_point());
                if !render_result.filter_mouse_actions
                    || render_result
                        .drawn_region
                        .is_inside(transformed_entrance_point)
                {
                    let transformed_event =
                        MouseEnterEvent::new(event.get_mouse(), transformed_entrance_point);
                    self.component
                        .on_mouse_enter(transformed_event, &mut self.buddy);
                }
            }
        }
    }

    fn mouse_leave(&mut self, event: MouseLeaveEvent) {
        if self.buddy.get_subscriptions().mouse_leave {
            if let Some(render_result) = self.buddy.get_last_render_result() {
                let transformed_exit_point = self.domain.transform(event.get_exit_point());
                if !render_result.filter_mouse_actions
                    || render_result.drawn_region.is_inside(transformed_exit_point)
                {
                    let transformed_event =
                        MouseLeaveEvent::new(event.get_mouse(), transformed_exit_point);
                    self.component
                        .on_mouse_leave(transformed_event, &mut self.buddy);
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
                let backup_region = RectangularDrawnRegion::new(0.0, 0.0, 1.0, 1.0);
                let reference_region = match render_result.filter_mouse_actions {
                    true => render_result.drawn_region.as_ref(),
                    false => &backup_region,
                };
                let intersection =
                    reference_region.find_line_intersection(transformed_from, transformed_to);
                match intersection {
                    LineIntersection::FullyOutside => {
                        // I don't need to do anything
                    }
                    LineIntersection::FullyInside => {
                        // Just pass a MouseMoveEvent
                        if sub_move {
                            let move_event = MouseMoveEvent::new(
                                event.get_mouse(),
                                transformed_from,
                                transformed_to,
                            );
                            self.component.on_mouse_move(move_event, &mut self.buddy);
                        }
                    }
                    LineIntersection::Enters { point } => {
                        // Pass a MouseEnterEvent and a MouseMoveEvent
                        if sub_enter {
                            let enter_event = MouseEnterEvent::new(event.get_mouse(), point);
                            self.component.on_mouse_enter(enter_event, &mut self.buddy);
                        }

                        // Note: the component might have subscribed during its on_mouse_enter
                        if self.buddy.get_subscriptions().mouse_move {
                            let move_event =
                                MouseMoveEvent::new(event.get_mouse(), point, transformed_to);
                            self.component.on_mouse_move(move_event, &mut self.buddy);
                        }
                    }
                    LineIntersection::Exits { point } => {
                        // Pass a MouseMoveEvent and a MouseLeaveEvent
                        if sub_move {
                            let move_event =
                                MouseMoveEvent::new(event.get_mouse(), transformed_from, point);
                            self.component.on_mouse_move(move_event, &mut self.buddy);
                        }

                        // Note: the component might have subscribed during its on_mouse_move
                        if self.buddy.get_subscriptions().mouse_leave {
                            let leave_event = MouseLeaveEvent::new(event.get_mouse(), point);
                            self.component.on_mouse_leave(leave_event, &mut self.buddy);
                        }
                    }
                    LineIntersection::Crosses { entrance, exit } => {
                        // Pass a MouseEnterEvent, MouseMoveEvent, and MouseLeaveEvent
                        if sub_enter {
                            let enter_event = MouseEnterEvent::new(event.get_mouse(), entrance);
                            self.component.on_mouse_enter(enter_event, &mut self.buddy);
                        }

                        // Note: the component might have subscribed during its on_mouse_enter
                        if self.buddy.get_subscriptions().mouse_move {
                            let move_event = MouseMoveEvent::new(event.get_mouse(), entrance, exit);
                            self.component.on_mouse_move(move_event, &mut self.buddy);
                        }

                        if self.buddy.get_subscriptions().mouse_leave {
                            let leave_event = MouseLeaveEvent::new(event.get_mouse(), exit);
                            self.component.on_mouse_leave(leave_event, &mut self.buddy);
                        }
                    }
                };
            }
        }
    }

    fn render(&mut self, renderer: &Renderer, force: bool) -> Option<RenderResult> {
        if force || self.buddy.did_request_render() {
            self.buddy.clear_render_request();

            let maybe_render_result = renderer.push_viewport(
                self.domain.get_min_x(),
                self.domain.get_min_y(),
                self.domain.get_max_x(),
                self.domain.get_max_y(),
                || self.component.render(renderer, &mut self.buddy, force),
            );

            if let Some(render_result) = maybe_render_result {
                if render_result.is_err() {
                    return Some(render_result);
                }

                let good_result = render_result.unwrap();
                self.buddy.set_last_render_result(good_result.clone());
                Some(Ok(good_result))
            } else {
                None
            }
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
                _renderer: &Renderer,
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
        menu.render(
            &test_renderer(RenderRegion::between(0, 0, 10, 10)),
            &mut buddy,
            false,
        )
        .unwrap();
        assert_eq!(1, counter1.get());
        assert_eq!(1, counter2.get());

        // But they should be attached only once
        menu.render(
            &test_renderer(RenderRegion::between(0, 0, 10, 10)),
            &mut buddy,
            false,
        )
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
                _renderer: &Renderer,
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
                _renderer: &Renderer,
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
        application.render(&test_renderer(RenderRegion::between(0, 0, 10, 10)), false);
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
                _renderer: &Renderer,
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
                _renderer: &Renderer,
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
        menu.render(&test_renderer(render_region), &mut buddy, false)
            .unwrap();
        assert_eq!(1, click_counter.get());

        // But the menu shouldn't request another one until we click the component
        assert!(!buddy.did_request_render());

        // So let's click it and render again
        let hit_click =
            MouseClickEvent::new(Mouse::new(0), Point::new(0.2, 0.2), MouseButton::primary());
        menu.on_mouse_click(hit_click, &mut buddy);
        assert!(buddy.did_request_render());
        buddy.clear_render_request();
        menu.render(&test_renderer(render_region), &mut buddy, false)
            .unwrap();
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
        menu.render(&test_renderer(render_region), &mut buddy, true)
            .unwrap();
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
        menu.render(&test_renderer(render_region), &mut buddy, false)
            .unwrap();
        // And it should have requested a redraw right away
        assert!(buddy.did_request_render());
        assert_eq!(3, click_counter.get());
        assert_eq!(1, busy_counter.get());

        // Again, only the busy component should render
        buddy.clear_render_request();
        menu.render(&test_renderer(render_region), &mut buddy, false)
            .unwrap();
        // And it should have requested a redraw again
        assert!(buddy.did_request_render());
        assert_eq!(3, click_counter.get());
        assert_eq!(2, busy_counter.get());

        // When we force a render, both components should be redrawn
        buddy.clear_render_request();
        menu.render(&test_renderer(render_region), &mut buddy, true)
            .unwrap();
        // Like usual, it should request a next render
        assert!(buddy.did_request_render());
        assert_eq!(4, click_counter.get());
        assert_eq!(3, busy_counter.get());

        // But when we render without force again, only the busy one should be drawn
        buddy.clear_render_request();
        menu.render(&test_renderer(render_region), &mut buddy, false)
            .unwrap();
        assert!(buddy.did_request_render());
        assert_eq!(4, click_counter.get());
        assert_eq!(4, busy_counter.get());

        // Unless we click it...
        menu.on_mouse_click(hit_click, &mut buddy);
        menu.render(&test_renderer(render_region), &mut buddy, false)
            .unwrap();
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
            _renderer: &Renderer,
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

        let result = menu
            .render(&test_renderer(render_region), &mut buddy, false)
            .unwrap();
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
        let result = menu
            .render(&test_renderer(render_region), &mut buddy, false)
            .unwrap();
        // This time, it should only have drawn the first component
        {
            let region = &result.drawn_region;
            assert!(region.is_inside(Point::new(0.0, 0.0)));
            assert!(region.is_inside(Point::new(0.5, 0.5)));
            assert!(!region.is_inside(Point::new(0.6, 0.6)));
        }

        menu.on_mouse_click(click2, &mut buddy);
        let result = menu
            .render(&test_renderer(render_region), &mut buddy, false)
            .unwrap();
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
        let result = menu
            .render(&test_renderer(render_region), &mut buddy, true)
            .unwrap();
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
        let result = menu
            .render(&test_renderer(render_region), &mut buddy, false)
            .unwrap();
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
                _renderer: &Renderer,
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
        menu.render(
            &test_renderer(RenderRegion::between(0, 0, 120, 10)),
            &mut buddy,
            false,
        )
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
            press_counter: Rc<Cell<u8>>,
            release_counter: Rc<Cell<u8>>,
        }

        impl Component for SubscribingComponent {
            fn on_attach(&mut self, _buddy: &mut dyn ComponentBuddy) {}

            fn render(
                &mut self,
                _renderer: &Renderer,
                buddy: &mut dyn ComponentBuddy,
                _force: bool,
            ) -> RenderResult {
                if self.should_subscribe.get() {
                    buddy.subscribe_mouse_click();
                    buddy.subscribe_mouse_click_out();
                    buddy.subscribe_mouse_press();
                    buddy.subscribe_mouse_release();
                }
                if self.should_unsubscribe.get() {
                    buddy.unsubscribe_mouse_click();
                    buddy.unsubscribe_mouse_click_out();
                    buddy.unsubscribe_mouse_press();
                    buddy.unsubscribe_mouse_release();
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

            fn on_mouse_press(&mut self, _event: MousePressEvent, _buddy: &mut dyn ComponentBuddy) {
                self.press_counter.set(self.press_counter.get() + 1);
            }

            fn on_mouse_release(
                &mut self,
                _event: MouseReleaseEvent,
                _buddy: &mut dyn ComponentBuddy,
            ) {
                self.release_counter.set(self.release_counter.get() + 1);
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
        let press_count1 = Rc::new(Cell::new(0));
        let press_count2 = Rc::new(Cell::new(0));
        let release_count1 = Rc::new(Cell::new(0));
        let release_count2 = Rc::new(Cell::new(0));

        let buddy = root_buddy();
        let region = RenderRegion::between(0, 10, 20, 30);
        let mut menu = SimpleFlatMenu::new(None);
        menu.add_component(
            Box::new(SubscribingComponent {
                should_subscribe: Rc::clone(&subscribe1),
                should_unsubscribe: Rc::clone(&unsubscribe1),
                click_counter: Rc::clone(&click_count1),
                click_out_counter: Rc::clone(&click_out_count1),
                press_counter: Rc::clone(&press_count1),
                release_counter: Rc::clone(&release_count1),
            }),
            ComponentDomain::between(0.0, 0.0, 0.5, 0.5),
        );
        menu.add_component(
            Box::new(SubscribingComponent {
                should_subscribe: Rc::clone(&subscribe2),
                should_unsubscribe: Rc::clone(&unsubscribe2),
                click_counter: Rc::clone(&click_count2),
                click_out_counter: Rc::clone(&click_out_count2),
                press_counter: Rc::clone(&press_count2),
                release_counter: Rc::clone(&release_count2),
            }),
            ComponentDomain::between(0.5, 0.5, 1.0, 1.0),
        );

        let menu_cell = Rc::new(RefCell::new(menu));
        let buddy_cell = Rc::new(RefCell::new(buddy));
        let do_render = || {
            let mut menu = menu_cell.borrow_mut();
            let mut buddy = buddy_cell.borrow_mut();
            menu.render(&test_renderer(region), &mut *buddy, true)
                .unwrap();
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
        let fire_press = || {
            let mut menu = menu_cell.borrow_mut();
            let mut buddy = buddy_cell.borrow_mut();
            menu.on_mouse_press(
                MousePressEvent::new(Mouse::new(0), Point::new(0.2, 0.2), MouseButton::primary()),
                &mut *buddy,
            );
            menu.on_mouse_press(
                MousePressEvent::new(Mouse::new(0), Point::new(0.8, 0.8), MouseButton::primary()),
                &mut *buddy,
            );
        };
        let fire_release = || {
            let mut menu = menu_cell.borrow_mut();
            let mut buddy = buddy_cell.borrow_mut();
            menu.on_mouse_release(
                MouseReleaseEvent::new(Mouse::new(0), Point::new(0.2, 0.2), MouseButton::primary()),
                &mut *buddy,
            );
            menu.on_mouse_release(
                MouseReleaseEvent::new(Mouse::new(0), Point::new(0.8, 0.8), MouseButton::primary()),
                &mut *buddy,
            );
        };

        let check_values = |click1: u8,
                            click_out1: u8,
                            press1: u8,
                            release1: u8,
                            click2: u8,
                            click_out2: u8,
                            press2: u8,
                            release2: u8| {
            assert_eq!(click1, click_count1.get());
            assert_eq!(click_out1, click_out_count1.get());
            assert_eq!(press1, press_count1.get());
            assert_eq!(release1, release_count1.get());
            assert_eq!(click2, click_count2.get());
            assert_eq!(click_out2, click_out_count2.get());
            assert_eq!(press2, press_count2.get());
            assert_eq!(release2, release_count2.get());
        };

        // No subscriptions yet, so these events should be ignored
        do_render();
        fire_click();
        fire_click_out();
        check_values(0, 0, 0, 0, 0, 0, 0, 0);

        // Lets subscribe the first component
        subscribe1.set(true);
        do_render();
        fire_click();
        check_values(1, 1, 0, 0, 0, 0, 0, 0);

        // Now the second one as well
        subscribe2.set(true);
        do_render();
        fire_click_out();
        fire_press();
        check_values(1, 2, 1, 0, 0, 1, 1, 0);

        // Nah, let's cancel the subscription for the second one
        unsubscribe2.set(true);
        do_render();
        fire_click();
        fire_release();
        check_values(2, 3, 1, 1, 0, 1, 1, 0);

        // This is not fair... lets cancel the first one as well
        unsubscribe1.set(true);
        do_render();
        fire_click_out();
        fire_release();
        fire_press();
        check_values(2, 3, 1, 1, 0, 1, 1, 0);

        // Lets give the second one a comeback
        subscribe2.set(true);
        do_render();
        fire_click();
        fire_click_out();
        fire_press();
        fire_release();
        check_values(2, 3, 1, 1, 1, 3, 2, 1);

        // Let's stop
        unsubscribe2.set(true);
        do_render();
        fire_click();
        fire_click_out();
        check_values(2, 3, 1, 1, 1, 3, 2, 1);
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

        fn render(
            &mut self,
            _renderer: &Renderer,
            _buddy: &mut dyn ComponentBuddy,
            _force: bool,
        ) -> RenderResult {
            Ok(RenderResultStruct {
                filter_mouse_actions: self.should_filter_mouse_actions.get(),
                drawn_region: Box::new(RectangularDrawnRegion::new(0.2, 0.2, 0.8, 0.8)),
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
            mouse_leave_log: Rc::clone(&leave_log1),
        };
        let component2 = MouseMotionComponent {
            should_filter_mouse_actions: Rc::new(Cell::new(true)),
            mouse_move_log: Rc::new(RefCell::new(Vec::new())),
            mouse_enter_log: Rc::clone(&enter_log2),
            mouse_leave_log: Rc::clone(&leave_log2),
        };

        let mut buddy = root_buddy();
        let mut menu = SimpleFlatMenu::new(None);
        menu.on_attach(&mut buddy);
        menu.add_component(
            Box::new(component1),
            ComponentDomain::between(0.1, 0.1, 0.4, 0.9),
        );
        menu.add_component(
            Box::new(component2),
            ComponentDomain::between(0.6, 0.1, 0.9, 0.9),
        );

        let miss_enter_event = MouseEnterEvent::new(Mouse::new(0), Point::new(0.5, 0.5));
        let miss_leave_event = MouseLeaveEvent::new(Mouse::new(0), Point::new(0.5, 0.5));
        let edge_enter_event = MouseEnterEvent::new(Mouse::new(0), Point::new(0.65, 0.5));
        let edge_leave_event = MouseLeaveEvent::new(Mouse::new(0), Point::new(0.65, 0.5));
        let hit_enter_event = MouseEnterEvent::new(Mouse::new(0), Point::new(0.75, 0.5));
        let hit_leave_event = MouseLeaveEvent::new(Mouse::new(0), Point::new(0.75, 0.5));
        let render_region = RenderRegion::between(1, 2, 3, 4);

        // Nothing should happen before the first render
        menu.on_mouse_enter(hit_enter_event, &mut buddy);
        menu.on_mouse_leave(hit_leave_event, &mut buddy);

        // So let's render
        menu.render(&test_renderer(render_region), &mut buddy, false)
            .unwrap();

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
        assert!(enter_log2[0]
            .get_entrance_point()
            .nearly_equal(Point::new(0.5, 0.5)));

        let leave_log2 = leave_log2.borrow();
        assert_eq!(1, leave_log2.len());
        assert!(leave_log2[0]
            .get_exit_point()
            .nearly_equal(Point::new(0.5, 0.5)));
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
        menu.add_component(
            Box::new(MouseMotionComponent {
                should_filter_mouse_actions: Rc::new(Cell::new(true)),
                mouse_move_log: Rc::clone(&move_logs[0]),
                mouse_enter_log: Rc::clone(&enter_logs[0]),
                mouse_leave_log: Rc::clone(&leave_logs[0]),
            }),
            ComponentDomain::between(0.0, 0.0, 0.25, 0.25),
        );

        // The inner bottom-left component
        menu.add_component(
            Box::new(MouseMotionComponent {
                should_filter_mouse_actions: Rc::new(Cell::new(false)),
                mouse_move_log: Rc::clone(&move_logs[1]),
                mouse_enter_log: Rc::clone(&enter_logs[1]),
                mouse_leave_log: Rc::clone(&leave_logs[1]),
            }),
            ComponentDomain::between(0.25, 0.25, 0.5, 0.5),
        );

        // The inner top-right component
        menu.add_component(
            Box::new(MouseMotionComponent {
                should_filter_mouse_actions: Rc::new(Cell::new(true)),
                mouse_move_log: Rc::clone(&move_logs[2]),
                mouse_enter_log: Rc::clone(&enter_logs[2]),
                mouse_leave_log: Rc::clone(&leave_logs[2]),
            }),
            ComponentDomain::between(0.5, 0.5, 0.75, 0.75),
        );

        // The outer top-right component
        menu.add_component(
            Box::new(MouseMotionComponent {
                should_filter_mouse_actions: Rc::new(Cell::new(true)),
                mouse_move_log: Rc::clone(&move_logs[4]),
                mouse_enter_log: Rc::clone(&enter_logs[4]),
                mouse_leave_log: Rc::clone(&leave_logs[4]),
            }),
            ComponentDomain::between(0.75, 0.75, 1.0, 1.0),
        );

        // This component should be missed entirely
        menu.add_component(
            Box::new(MouseMotionComponent {
                should_filter_mouse_actions: Rc::new(Cell::new(false)),
                mouse_move_log: Rc::clone(&move_logs[3]),
                mouse_enter_log: Rc::clone(&enter_logs[3]),
                mouse_leave_log: Rc::clone(&leave_logs[3]),
            }),
            ComponentDomain::between(0.5, 0.0, 0.75, 0.25),
        );

        menu.render(
            &test_renderer(RenderRegion::between(0, 0, 20, 30)),
            &mut buddy,
            false,
        )
        .unwrap();

        let mouse = Mouse::new(3);
        let entrance_x = 0.25 * 0.25;
        let entrance_y = 0.25 * 0.25;
        let entrance = Point::new(entrance_x, entrance_y);
        let exit_x = 1.0 - entrance_x;
        let exit_y = 1.0 - entrance_y;
        let exit = Point::new(exit_x, exit_y);

        let enter_event = MouseEnterEvent::new(mouse, entrance);
        let move_event = MouseMoveEvent::new(mouse, entrance, exit);
        let leave_event = MouseLeaveEvent::new(mouse, exit);
        menu.on_mouse_enter(enter_event, &mut buddy);
        menu.on_mouse_move(move_event, &mut buddy);
        menu.on_mouse_leave(leave_event, &mut buddy);

        // Time to check the results...

        // But first some helper functions
        let eq_mouse_move =
            |enter_x: f32, enter_y: f32, exit_x: f32, exit_y: f32, event: &MouseMoveEvent| {
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
                &mut self,
                _renderer: &Renderer,
                buddy: &mut dyn ComponentBuddy,
                _force: bool,
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
            ComponentDomain::between(0.0, 0.0, 0.5, 0.5),
        );

        let mut try_combination = |mouse_move: bool, mouse_enter: bool, mouse_leave: bool| {
            sub_mouse_move.set(mouse_move);
            sub_mouse_enter.set(mouse_enter);
            sub_mouse_leave.set(mouse_leave);
            menu.render(
                &test_renderer(RenderRegion::between(0, 1, 4, 7)),
                &mut buddy,
                true,
            )
            .unwrap();
            let mouse = Mouse::new(2);
            let original_enter_event1 = MouseEnterEvent::new(mouse, Point::new(0.1, 0.6));
            let original_enter_event2 =
                MouseMoveEvent::new(mouse, Point::new(0.1, 0.6), Point::new(0.1, 0.25));
            let original_move_event =
                MouseMoveEvent::new(mouse, Point::new(0.1, 0.25), Point::new(0.4, 0.25));
            let original_leave_event1 =
                MouseMoveEvent::new(mouse, Point::new(0.4, 0.25), Point::new(0.4, 0.6));
            let original_leave_event2 = MouseLeaveEvent::new(mouse, Point::new(0.4, 0.6));
            let transformed_enter_event1 = MouseEnterEvent::new(mouse, Point::new(0.2, 1.0));
            let transformed_enter_event2 =
                MouseMoveEvent::new(mouse, Point::new(0.2, 1.0), Point::new(0.2, 0.5));
            let transformed_move_event =
                MouseMoveEvent::new(mouse, Point::new(0.2, 0.5), Point::new(0.8, 0.5));
            let transformed_leave_event1 =
                MouseMoveEvent::new(mouse, Point::new(0.8, 0.5), Point::new(0.8, 1.0));
            let transformed_leave_event2 = MouseLeaveEvent::new(mouse, Point::new(0.8, 1.0));

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
                    assert!(expected
                        .get_entrance_point()
                        .nearly_equal(actual.get_entrance_point()));
                };

                assert_eq!(1, enter_log.len());
                enter_event_eq(&transformed_enter_event1, &enter_log[0]);
            } else {
                assert!(enter_log.is_empty());
            }

            if mouse_leave {
                let leave_event_eq = |expected: &MouseLeaveEvent, actual: &MouseLeaveEvent| {
                    assert_eq!(expected.get_mouse(), actual.get_mouse());
                    assert!(expected
                        .get_exit_point()
                        .nearly_equal(actual.get_exit_point()));
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

        for _counter in 0..2 {
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
                buddy.subscribe_mouse_press();
                buddy.subscribe_mouse_release();
                buddy.subscribe_mouse_move();
                buddy.subscribe_mouse_enter();
                buddy.subscribe_mouse_leave();
            }

            fn render(
                &mut self,
                _renderer: &Renderer,
                _buddy: &mut dyn ComponentBuddy,
                _force: bool,
            ) -> RenderResult {
                entire_render_result()
            }
        }

        let mut menu = SimpleFlatMenu::new(None);
        let mut buddy = root_buddy();
        menu.on_attach(&mut buddy);
        menu.add_component(
            Box::new(CuriousComponent {}),
            ComponentDomain::between(0.3, 0.6, 1.0, 0.9),
        );

        // The menu should have subscribed to all events
        let subs = buddy.get_subscriptions();
        assert!(subs.mouse_click);
        assert!(subs.mouse_click_out);
        assert!(subs.mouse_press);
        assert!(subs.mouse_release);
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

            fn render(
                &mut self,
                _renderer: &Renderer,
                buddy: &mut dyn ComponentBuddy,
                _force: bool,
            ) -> RenderResult {
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
                call_counter: Rc::clone(&call_counter),
            }),
            ComponentDomain::between(0.1, 0.2, 0.3, 0.4),
        );

        let mut application = Application::new(Box::new(menu));

        let region = RenderRegion::with_size(1, 2, 3, 4);

        // The mouses should be empty initially
        application.render(&test_renderer(region), true);

        let enter_event =
            |mouse_id: u16| MouseEnterEvent::new(Mouse::new(mouse_id), Point::new(0.2, 0.3));
        let leave_event =
            |mouse_id: u16| MouseLeaveEvent::new(Mouse::new(mouse_id), Point::new(0.2, 0.3));
        let mouse_vec = |ids: &[u16]| ids.iter().map(|id| Mouse::new(*id)).collect();

        // Add the first mouse
        application.fire_mouse_enter_event(enter_event(123));
        expected_mouses.replace(mouse_vec(&[123]));
        application.render(&test_renderer(region), true);

        // Add the second mouse
        application.fire_mouse_enter_event(enter_event(1));
        expected_mouses.replace(mouse_vec(&[123, 1]));
        application.render(&test_renderer(region), true);

        // Remove the first mouse
        application.fire_mouse_leave_event(leave_event(123));
        expected_mouses.replace(mouse_vec(&[1]));
        application.render(&test_renderer(region), true);

        // Add the first mouse back, and add yet another mouse
        application.fire_mouse_enter_event(enter_event(123));
        application.fire_mouse_enter_event(enter_event(8));
        expected_mouses.replace(mouse_vec(&[1, 123, 8]));
        application.render(&test_renderer(region), true);

        // Remove all mouses
        application.fire_mouse_leave_event(leave_event(123));
        application.fire_mouse_leave_event(leave_event(8));
        application.fire_mouse_leave_event(leave_event(1));
        expected_mouses.replace(mouse_vec(&[]));
        application.render(&test_renderer(region), true);

        assert_eq!(6, call_counter.get());
    }

    #[test]
    fn test_buddy_get_local_mouses_and_positions() {
        struct LocalMouse {
            mouse: Mouse,
            position: Point,
        }

        struct LocalMouseCheckComponent {
            expected_mouses: Rc<RefCell<Vec<LocalMouse>>>,
        }

        impl Component for LocalMouseCheckComponent {
            fn on_attach(&mut self, _buddy: &mut dyn ComponentBuddy) {}

            fn render(
                &mut self,
                _renderer: &Renderer,
                buddy: &mut dyn ComponentBuddy,
                _force: bool,
            ) -> RenderResult {
                let local_mouses = buddy.get_local_mouses();
                let expected_mouses = self.expected_mouses.borrow();
                assert_eq!(expected_mouses.len(), local_mouses.len());

                // Test that all local mouses are present and have the right position
                'local_outer: for mouse in &local_mouses {
                    for entry in &*expected_mouses {
                        if entry.mouse == *mouse {
                            assert!(entry
                                .position
                                .nearly_equal(buddy.get_mouse_position(*mouse).unwrap()));
                            continue 'local_outer;
                        }
                    }
                    panic!("Expected mouse {:?}, but didn't find its entry", mouse);
                }

                // Test that the other mouses do not have a position
                'all_outer: for mouse in buddy.get_all_mouses() {
                    for local_mouse in &local_mouses {
                        if *local_mouse == mouse {
                            continue 'all_outer;
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
        let set1 = |expected: Vec<LocalMouse>| set(&expected_mouses1, expected);
        let set2 = |expected: Vec<LocalMouse>| set(&expected_mouses2, expected);

        let mut menu = SimpleFlatMenu::new(None);
        menu.add_component(
            Box::new(LocalMouseCheckComponent {
                expected_mouses: Rc::clone(&expected_mouses1),
            }),
            ComponentDomain::between(0.2, 0.0, 0.5, 0.7),
        );
        menu.add_component(
            Box::new(LocalMouseCheckComponent {
                expected_mouses: Rc::clone(&expected_mouses2),
            }),
            ComponentDomain::between(0.5, 0.5, 1.0, 1.0),
        );

        let mut application = Application::new(Box::new(menu));
        let region = RenderRegion::between(10, 20, 30, 40);
        application.render(&test_renderer(region), true);

        // Start with 1 mouse, and spawn it in the middle of the first component
        let mouse1 = Mouse::new(6);
        application.fire_mouse_enter_event(MouseEnterEvent::new(mouse1, Point::new(0.35, 0.35)));
        set1(vec![LocalMouse {
            mouse: mouse1,
            position: Point::new(0.5, 0.5),
        }]);
        set2(vec![]);
        application.render(&test_renderer(region), true);

        // Move the mouse to the other component
        application.fire_mouse_move_event(MouseMoveEvent::new(
            mouse1,
            Point::new(0.35, 0.35),
            Point::new(0.6, 0.9),
        ));
        set1(vec![]);
        set2(vec![LocalMouse {
            mouse: mouse1,
            position: Point::new(0.2, 0.8),
        }]);
        application.render(&test_renderer(region), true);

        // Move the mouse away from both components
        application.fire_mouse_move_event(MouseMoveEvent::new(
            mouse1,
            Point::new(0.6, 0.9),
            Point::new(0.1, 0.1),
        ));
        set1(vec![]);
        set2(vec![]);
        application.render(&test_renderer(region), true);

        // Introduce the second mouse
        let mouse2 = Mouse::new(120);
        application.fire_mouse_enter_event(MouseEnterEvent::new(mouse2, Point::new(0.1, 0.1)));
        // Neither of the mouses is inside any of the components
        application.render(&test_renderer(region), true);

        // Move the second mouse to the second component
        application.fire_mouse_move_event(MouseMoveEvent::new(
            mouse2,
            Point::new(0.1, 0.1),
            Point::new(0.7, 0.8),
        ));
        set1(vec![]);
        set2(vec![LocalMouse {
            mouse: mouse2,
            position: Point::new(0.4, 0.6),
        }]);
        application.render(&test_renderer(region), true);

        // Move the first mouse to the first component
        application.fire_mouse_move_event(MouseMoveEvent::new(
            mouse1,
            Point::new(0.1, 0.1),
            Point::new(0.35, 0.35),
        ));
        set1(vec![LocalMouse {
            mouse: mouse1,
            position: Point::new(0.5, 0.5),
        }]);
        set2(vec![LocalMouse {
            mouse: mouse2,
            position: Point::new(0.4, 0.6),
        }]);
        application.render(&test_renderer(region), true);

        // Move the first mouse to the second component
        application.fire_mouse_move_event(MouseMoveEvent::new(
            mouse1,
            Point::new(0.35, 0.35),
            Point::new(0.8, 0.7),
        ));
        set1(vec![]);
        set2(vec![
            LocalMouse {
                mouse: mouse1,
                position: Point::new(0.6, 0.4),
            },
            LocalMouse {
                mouse: mouse2,
                position: Point::new(0.4, 0.6),
            },
        ]);
        application.render(&test_renderer(region), true);

        // Remove the second mouse
        application.fire_mouse_leave_event(MouseLeaveEvent::new(mouse2, Point::new(0.7, 0.8)));
        set1(vec![]);
        set2(vec![LocalMouse {
            mouse: mouse1,
            position: Point::new(0.6, 0.4),
        }]);
        application.render(&test_renderer(region), true);
    }

    #[test]
    fn test_render_viewports_and_scissors() {
        struct ViewportTestComponent {
            expected_viewport: Rc<Cell<RenderRegion>>,
            render_counter: Rc<Cell<u8>>,
        }
        impl Component for ViewportTestComponent {
            fn on_attach(&mut self, _buddy: &mut dyn ComponentBuddy) {}

            fn render(
                &mut self,
                renderer: &Renderer,
                _buddy: &mut dyn ComponentBuddy,
                _force: bool,
            ) -> RenderResult {
                self.render_counter.set(self.render_counter.get() + 1);
                assert_eq!(self.expected_viewport.get(), renderer.get_viewport());
                assert_eq!(self.expected_viewport.get(), renderer.get_scissor());
                entire_render_result()
            }
        }

        let mut renderer = test_renderer(RenderRegion::with_size(0, 0, 100, 100));
        let mut buddy = root_buddy();
        let mut menu = SimpleFlatMenu::new(None);

        let render_counter = Rc::new(Cell::new(0));

        // I will assign proper values later
        let viewport1 = Rc::new(Cell::new(RenderRegion::with_size(1, 2, 3, 4)));
        let viewport2 = Rc::new(Cell::new(RenderRegion::with_size(1, 2, 3, 4)));

        menu.add_component(
            Box::new(ViewportTestComponent {
                render_counter: Rc::clone(&render_counter),
                expected_viewport: Rc::clone(&viewport1),
            }),
            ComponentDomain::between(0.0, 0.0, 0.5, 0.5),
        );
        menu.add_component(
            Box::new(ViewportTestComponent {
                render_counter: Rc::clone(&render_counter),
                expected_viewport: Rc::clone(&viewport2),
            }),
            ComponentDomain::between(0.2, 0.5, 0.9, 1.0),
        );

        viewport1.set(RenderRegion::between(0, 0, 50, 50));
        viewport2.set(RenderRegion::between(20, 50, 90, 100));
        menu.render(&renderer, &mut buddy, true).unwrap();

        renderer.reset_viewport(RenderRegion::between(100, 200, 300, 400));
        viewport1.set(RenderRegion::between(100, 200, 200, 300));
        viewport2.set(RenderRegion::between(140, 300, 280, 400));
        menu.render(&renderer, &mut buddy, true).unwrap();

        // Check that the render methods were actually called twice per component
        assert_eq!(4, render_counter.get());
    }

    #[test]
    fn test_render_with_custom_scissor() {
        struct ScissorTestComponent {
            expected_viewport: Rc<Cell<RenderRegion>>,
            expected_scissor: Rc<Cell<RenderRegion>>,
            render_counter: Rc<Cell<u8>>,
        }
        impl Component for ScissorTestComponent {
            fn on_attach(&mut self, _buddy: &mut dyn ComponentBuddy) {}

            fn render(
                &mut self,
                renderer: &Renderer,
                _buddy: &mut dyn ComponentBuddy,
                _force: bool,
            ) -> RenderResult {
                self.render_counter.set(self.render_counter.get() + 1);
                assert_eq!(self.expected_viewport.get(), renderer.get_viewport());
                assert_eq!(self.expected_scissor.get(), renderer.get_scissor());
                entire_render_result()
            }
        }

        let counter_left = Rc::new(Cell::new(0));
        let counter_middle = Rc::new(Cell::new(0));
        let counter_right = Rc::new(Cell::new(0));

        let viewport_left = Rc::new(Cell::new(RenderRegion::between(10, 10, 20, 20)));
        let scissor_left = Rc::new(Cell::new(RenderRegion::between(10, 10, 20, 20)));
        let viewport_middle = Rc::new(Cell::new(RenderRegion::between(40, 40, 60, 60)));
        let scissor_middle = Rc::new(Cell::new(RenderRegion::between(40, 40, 60, 60)));
        let viewport_right = Rc::new(Cell::new(RenderRegion::between(80, 80, 90, 90)));
        let scissor_right = Rc::new(Cell::new(RenderRegion::between(80, 80, 90, 90)));

        let mut menu = SimpleFlatMenu::new(None);
        menu.add_component(
            Box::new(ScissorTestComponent {
                expected_viewport: Rc::clone(&viewport_left),
                expected_scissor: Rc::clone(&scissor_left),
                render_counter: Rc::clone(&counter_left),
            }),
            ComponentDomain::between(0.1, 0.1, 0.2, 0.2),
        );
        menu.add_component(
            Box::new(ScissorTestComponent {
                expected_viewport: Rc::clone(&viewport_middle),
                expected_scissor: Rc::clone(&scissor_middle),
                render_counter: Rc::clone(&counter_middle),
            }),
            ComponentDomain::between(0.4, 0.4, 0.6, 0.6),
        );
        menu.add_component(
            Box::new(ScissorTestComponent {
                expected_viewport: Rc::clone(&viewport_right),
                expected_scissor: Rc::clone(&scissor_right),
                render_counter: Rc::clone(&counter_right),
            }),
            ComponentDomain::between(0.8, 0.8, 0.9, 0.9),
        );

        let renderer = test_renderer(RenderRegion::with_size(0, 0, 100, 100));
        let mut buddy = root_buddy();

        // First try without scissor
        menu.render(&renderer, &mut buddy, true).unwrap();
        assert_eq!(1, counter_left.get());
        assert_eq!(1, counter_middle.get());
        assert_eq!(1, counter_right.get());

        // Now with a scissor in the left half
        scissor_middle.set(RenderRegion::between(40, 40, 50, 60));
        renderer.push_scissor(0.0, 0.0, 0.5, 1.0, || {
            menu.render(&renderer, &mut buddy, true).unwrap();
        });
        assert_eq!(2, counter_left.get());
        assert_eq!(2, counter_middle.get());
        assert_eq!(1, counter_right.get());
    }

    #[test]
    fn test_buddy_change_menu() {
        struct NewMenuComponent {
            counter: Rc<Cell<u8>>,
        }

        impl Component for NewMenuComponent {
            fn on_attach(&mut self, buddy: &mut dyn ComponentBuddy) {
                buddy.subscribe_mouse_click();
                self.counter.set(5);
            }

            fn render(
                &mut self,
                _renderer: &Renderer,
                _buddy: &mut dyn ComponentBuddy,
                _force: bool,
            ) -> RenderResult {
                self.counter.set(self.counter.get() * 3);
                entire_render_result()
            }

            fn on_mouse_click(&mut self, _event: MouseClickEvent, _buddy: &mut dyn ComponentBuddy) {
                self.counter.set(self.counter.get() + 1);
            }
        }

        struct ChangeMenuComponent {
            counter: Rc<Cell<u8>>,
            new_counter: Rc<Cell<u8>>,
        }

        impl Component for ChangeMenuComponent {
            fn on_attach(&mut self, buddy: &mut dyn ComponentBuddy) {
                buddy.subscribe_mouse_click();
                self.counter.set(2);
            }

            fn render(
                &mut self,
                _renderer: &Renderer,
                _buddy: &mut dyn ComponentBuddy,
                _force: bool,
            ) -> RenderResult {
                self.counter.set(self.counter.get() * 3);
                entire_render_result()
            }

            fn on_mouse_click(&mut self, _event: MouseClickEvent, buddy: &mut dyn ComponentBuddy) {
                self.counter.set(self.counter.get() + 1);
                let new_counter = Rc::clone(&self.new_counter);
                buddy.change_menu(Box::new(move |_old_menu| {
                    Box::new(NewMenuComponent {
                        counter: new_counter,
                    })
                }));
            }
        }

        let counter = Rc::new(Cell::new(0));
        let new_counter = Rc::new(Cell::new(0));

        let mut menu = SimpleFlatMenu::new(None);
        menu.add_component(
            Box::new(ChangeMenuComponent {
                counter: Rc::clone(&counter),
                new_counter: Rc::clone(&new_counter),
            }),
            ComponentDomain::between(0.3, 0.3, 0.7, 0.7),
        );

        let mut application = Application::new(Box::new(menu));

        let region = RenderRegion::with_size(1, 2, 3, 4);
        let renderer = test_renderer(region);

        assert_eq!(2, counter.get());
        application.render(&renderer, false);
        assert_eq!(6, counter.get());
        assert_eq!(0, new_counter.get());

        let click_event =
            MouseClickEvent::new(Mouse::new(3), Point::new(0.4, 0.4), MouseButton::primary());

        application.fire_mouse_click_event(click_event);
        assert_eq!(7, counter.get());
        assert_eq!(5, new_counter.get());

        application.render(&renderer, false);
        assert_eq!(7, counter.get());
        assert_eq!(15, new_counter.get());

        application.fire_mouse_click_event(click_event);
        assert_eq!(7, counter.get());
        assert_eq!(16, new_counter.get());
    }

    #[test]
    fn test_mouse_press_and_release() {
        struct PressReleaseComponent {
            press_counter: Rc<Cell<u8>>,
            release_counter: Rc<Cell<u8>>,
            expected_press_event: Rc<Cell<MousePressEvent>>,
            expected_release_event: Rc<Cell<MouseReleaseEvent>>,
            filter_mouse_actions: bool,
        }

        impl Component for PressReleaseComponent {
            fn on_attach(&mut self, buddy: &mut dyn ComponentBuddy) {
                buddy.subscribe_mouse_press();
                buddy.subscribe_mouse_release();
            }

            fn render(
                &mut self,
                _renderer: &Renderer,
                _buddy: &mut dyn ComponentBuddy,
                _force: bool,
            ) -> RenderResult {
                Ok(RenderResultStruct {
                    filter_mouse_actions: self.filter_mouse_actions,
                    drawn_region: Box::new(RectangularDrawnRegion::new(0.2, 0.2, 0.8, 0.8)),
                })
            }

            fn on_mouse_press(&mut self, event: MousePressEvent, _buddy: &mut dyn ComponentBuddy) {
                self.press_counter.set(self.press_counter.get() + 1);
                let expected = self.expected_press_event.get();
                assert_eq!(expected.get_mouse(), event.get_mouse());
                assert!(expected.get_point().nearly_equal(event.get_point()));
                assert_eq!(expected.get_button(), event.get_button());
            }

            fn on_mouse_release(
                &mut self,
                event: MouseReleaseEvent,
                _buddy: &mut dyn ComponentBuddy,
            ) {
                self.release_counter.set(self.release_counter.get() + 1);
                let expected = self.expected_release_event.get();
                assert_eq!(expected.get_mouse(), event.get_mouse());
                assert!(expected.get_point().nearly_equal(event.get_point()));
                assert_eq!(expected.get_button(), event.get_button());
            }
        }

        // The initial events don't matter, but need to be specified
        let dummy_press_event =
            MousePressEvent::new(Mouse::new(111), Point::new(1.1, 1.1), MouseButton::new(11));
        let dummy_release_event =
            MouseReleaseEvent::new(Mouse::new(222), Point::new(2.2, 2.2), MouseButton::new(22));

        let press_counter1 = Rc::new(Cell::new(0));
        let release_counter1 = Rc::new(Cell::new(0));
        let expected_press_event1 = Rc::new(Cell::new(dummy_press_event));
        let expected_release_event1 = Rc::new(Cell::new(dummy_release_event));

        let press_counter2 = Rc::new(Cell::new(0));
        let release_counter2 = Rc::new(Cell::new(0));
        let expected_press_event2 = Rc::new(Cell::new(dummy_press_event));
        let expected_release_event2 = Rc::new(Cell::new(dummy_release_event));

        let mut buddy = root_buddy();
        let mut menu = SimpleFlatMenu::new(None);

        menu.add_component(
            Box::new(PressReleaseComponent {
                press_counter: Rc::clone(&press_counter1),
                release_counter: Rc::clone(&release_counter1),
                expected_press_event: Rc::clone(&expected_press_event1),
                expected_release_event: Rc::clone(&expected_release_event1),
                filter_mouse_actions: true,
            }),
            ComponentDomain::between(0.0, 0.0, 0.5, 0.5),
        );

        menu.add_component(
            Box::new(PressReleaseComponent {
                press_counter: Rc::clone(&press_counter2),
                release_counter: Rc::clone(&release_counter2),
                expected_press_event: Rc::clone(&expected_press_event2),
                expected_release_event: Rc::clone(&expected_release_event2),
                filter_mouse_actions: false,
            }),
            ComponentDomain::between(0.5, 0.5, 1.0, 1.0),
        );

        // Attaching and rendering should be done before sending events
        menu.on_attach(&mut buddy);
        menu.render(
            &test_renderer(RenderRegion::with_size(0, 0, 1000, 1000)),
            &mut buddy,
            false,
        )
        .unwrap();

        let check_counters = |press1: u8, release1: u8, press2: u8, release2: u8| {
            assert_eq!(press1, press_counter1.get());
            assert_eq!(release1, release_counter1.get());
            assert_eq!(press2, press_counter2.get());
            assert_eq!(release2, release_counter2.get());
        };

        // Miss these events on purpose:
        menu.on_mouse_press(
            MousePressEvent::new(Mouse::new(1), Point::new(0.2, 0.7), MouseButton::primary()),
            &mut buddy,
        );
        menu.on_mouse_release(
            MouseReleaseEvent::new(Mouse::new(1), Point::new(0.2, 0.7), MouseButton::primary()),
            &mut buddy,
        );
        check_counters(0, 0, 0, 0);

        // Press and release in the middle of both components
        expected_press_event1.set(MousePressEvent::new(
            Mouse::new(2),
            Point::new(0.5, 0.5),
            MouseButton::new(1),
        ));
        menu.on_mouse_press(
            MousePressEvent::new(Mouse::new(2), Point::new(0.25, 0.25), MouseButton::new(1)),
            &mut buddy,
        );
        expected_press_event2.set(MousePressEvent::new(
            Mouse::new(3),
            Point::new(0.5, 0.5),
            MouseButton::new(2),
        ));
        menu.on_mouse_press(
            MousePressEvent::new(Mouse::new(3), Point::new(0.75, 0.75), MouseButton::new(2)),
            &mut buddy,
        );
        expected_release_event1.set(MouseReleaseEvent::new(
            Mouse::new(2),
            Point::new(0.5, 0.5),
            MouseButton::new(1),
        ));
        menu.on_mouse_release(
            MouseReleaseEvent::new(Mouse::new(2), Point::new(0.25, 0.25), MouseButton::new(1)),
            &mut buddy,
        );
        expected_release_event2.set(MouseReleaseEvent::new(
            Mouse::new(3),
            Point::new(0.5, 0.5),
            MouseButton::new(2),
        ));
        menu.on_mouse_release(
            MouseReleaseEvent::new(Mouse::new(3), Point::new(0.75, 0.75), MouseButton::new(2)),
            &mut buddy,
        );
        check_counters(1, 1, 1, 1);

        // This time, press near bottom left corner and release near top right corner
        // Since the first component filters mouse actions, only the second component should notice this
        menu.on_mouse_press(
            MousePressEvent::new(Mouse::new(4), Point::new(0.05, 0.05), MouseButton::new(3)),
            &mut buddy,
        );
        menu.on_mouse_release(
            MouseReleaseEvent::new(Mouse::new(4), Point::new(0.45, 0.45), MouseButton::new(3)),
            &mut buddy,
        );
        expected_press_event2.set(MousePressEvent::new(
            Mouse::new(5),
            Point::new(0.1, 0.1),
            MouseButton::new(4),
        ));
        menu.on_mouse_press(
            MousePressEvent::new(Mouse::new(5), Point::new(0.55, 0.55), MouseButton::new(4)),
            &mut buddy,
        );
        expected_release_event2.set(MouseReleaseEvent::new(
            Mouse::new(5),
            Point::new(0.9, 0.9),
            MouseButton::new(4),
        ));
        menu.on_mouse_release(
            MouseReleaseEvent::new(Mouse::new(5), Point::new(0.95, 0.95), MouseButton::new(4)),
            &mut buddy,
        );
        check_counters(1, 1, 2, 2);
    }

    #[test]
    fn test_buddy_pressed_mouse_buttons() {
        struct MouseCheck {
            mouse: Mouse,
            button: MouseButton,
            result: Option<bool>,
        }

        impl MouseCheck {
            fn new(mouse: Mouse, button: MouseButton, result: Option<bool>) -> Self {
                Self {
                    mouse,
                    button,
                    result,
                }
            }
        }

        struct VecCheck {
            mouse: Mouse,
            buttons: Option<Vec<MouseButton>>,
        }

        impl VecCheck {
            fn new(mouse: Mouse, buttons: Option<Vec<MouseButton>>) -> Self {
                Self { mouse, buttons }
            }
        }

        struct MouseCheckComponent {
            checks: Rc<Cell<Vec<MouseCheck>>>,
            vec_checks: Rc<Cell<Vec<VecCheck>>>,
            render_counter: Rc<Cell<u8>>,
        }

        impl Component for MouseCheckComponent {
            fn on_attach(&mut self, _buddy: &mut dyn ComponentBuddy) {}

            fn render(
                &mut self,
                _renderer: &Renderer,
                buddy: &mut dyn ComponentBuddy,
                _force: bool,
            ) -> RenderResult {
                let checks = self.checks.take();
                for check in checks {
                    assert_eq!(
                        check.result,
                        buddy.is_mouse_button_down(check.mouse, check.button)
                    );
                }

                let vec_checks = self.vec_checks.take();
                for check in vec_checks {
                    assert_eq!(check.buttons, buddy.get_pressed_mouse_buttons(check.mouse));
                }

                self.render_counter.set(self.render_counter.get() + 1);
                entire_render_result()
            }
        }

        let counter1 = Rc::new(Cell::new(0));
        let counter2 = Rc::new(Cell::new(0));

        let checks1 = Rc::new(Cell::new(Vec::new()));
        let checks2 = Rc::new(Cell::new(Vec::new()));

        let vec_checks1 = Rc::new(Cell::new(Vec::new()));
        let vec_checks2 = Rc::new(Cell::new(Vec::new()));

        let mut menu = SimpleFlatMenu::new(None);
        menu.add_component(
            Box::new(MouseCheckComponent {
                checks: Rc::clone(&checks1),
                vec_checks: Rc::clone(&vec_checks1),
                render_counter: Rc::clone(&counter1),
            }),
            ComponentDomain::between(0.2, 0.5, 0.7, 0.7),
        );
        menu.add_component(
            Box::new(MouseCheckComponent {
                checks: Rc::clone(&checks2),
                vec_checks: Rc::clone(&vec_checks2),
                render_counter: Rc::clone(&counter2),
            }),
            ComponentDomain::between(0.8, 0.8, 1.0, 1.0),
        );

        let check_counters = |expected: u8| {
            assert_eq!(expected, counter1.get());
            assert_eq!(expected, counter2.get());
        };

        let mut application = Application::new(Box::new(menu));

        let renderer = test_renderer(RenderRegion::between(10, 20, 30, 40));

        // No mouse should be present initially
        checks1.set(vec![MouseCheck::new(
            Mouse::new(0),
            MouseButton::primary(),
            None,
        )]);
        vec_checks1.set(vec![VecCheck::new(Mouse::new(0), None)]);
        application.render(&renderer, true);
        check_counters(1);

        // Spawn a mouse on component 1, but don't press any buttons yet
        let mouse1 = Mouse::new(3);
        application.fire_mouse_enter_event(MouseEnterEvent::new(mouse1, Point::new(0.6, 0.6)));
        checks1.set(vec![MouseCheck::new(
            mouse1,
            MouseButton::primary(),
            Some(false),
        )]);
        checks2.set(vec![MouseCheck::new(mouse1, MouseButton::primary(), None)]);
        vec_checks1.set(vec![VecCheck::new(mouse1, Some(Vec::new()))]);
        vec_checks2.set(vec![VecCheck::new(mouse1, None)]);
        application.render(&renderer, true);
        check_counters(2);

        // Press a button
        let button1 = MouseButton::new(1);
        let button2 = MouseButton::new(2);
        application.fire_mouse_press_event(MousePressEvent::new(
            mouse1,
            Point::new(0.6, 0.6),
            button1,
        ));
        checks1.set(vec![
            MouseCheck::new(mouse1, button1, Some(true)),
            MouseCheck::new(mouse1, button2, Some(false)),
        ]);
        checks2.set(vec![
            MouseCheck::new(mouse1, button1, None),
            MouseCheck::new(mouse1, button2, None),
        ]);
        vec_checks1.set(vec![
            VecCheck::new(mouse1, Some(vec![button1])),
            VecCheck::new(Mouse::new(10), None),
        ]);
        application.render(&renderer, true);
        check_counters(3);

        // Move the mouse away
        application.fire_mouse_move_event(MouseMoveEvent::new(
            mouse1,
            Point::new(0.6, 0.6),
            Point::new(0.0, 0.0),
        ));
        checks1.set(vec![
            MouseCheck::new(mouse1, button1, None),
            MouseCheck::new(mouse1, button2, None),
        ]);
        checks2.set(vec![
            MouseCheck::new(mouse1, button1, None),
            MouseCheck::new(mouse1, button2, None),
        ]);
        vec_checks1.set(vec![VecCheck::new(mouse1, None)]);
        application.render(&renderer, true);
        check_counters(4);

        // Move the mouse to component 2
        application.fire_mouse_move_event(MouseMoveEvent::new(
            mouse1,
            Point::new(0.0, 0.0),
            Point::new(0.9, 0.9),
        ));
        checks1.set(vec![
            MouseCheck::new(mouse1, button1, None),
            MouseCheck::new(mouse1, button2, None),
        ]);
        checks2.set(vec![
            MouseCheck::new(mouse1, button1, Some(true)),
            MouseCheck::new(mouse1, button2, Some(false)),
        ]);
        vec_checks2.set(vec![
            VecCheck::new(mouse1, Some(vec![button1])),
            VecCheck::new(Mouse::new(10), None),
        ]);
        application.render(&renderer, true);
        check_counters(5);
    }
}
