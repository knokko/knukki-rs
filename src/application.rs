use crate::*;

use std::cell::RefCell;
use std::rc::Rc;

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
    root_buddy: RootComponentBuddy,

    mouse_store: Rc<RefCell<MouseStore>>,
}

impl Application {
    pub fn new(mut initial_root_component: Box<dyn Component>) -> Self {
        let mouse_store = Rc::new(RefCell::new(MouseStore::new()));

        let mut root_buddy = RootComponentBuddy::new();
        root_buddy.set_mouse_store(Rc::clone(&mouse_store));

        initial_root_component.on_attach(&mut root_buddy);
        // No need to call request_render, because the did_request_render field
        // of RootComponentBuddy starts as true
        let mut result = Self {
            root_component: initial_root_component,
            root_buddy,

            mouse_store,
        };
        result.work_after_events();
        result
    }

    fn work_after_events(&mut self) {
        if self.root_buddy.has_next_menu() {
            self.root_component.on_detach();

            // Work around because self.root_component must have some value at all times
            let mut replacement_helper: Box<dyn Component> = Box::new(DummyComponent {});
            std::mem::swap(&mut replacement_helper, &mut self.root_component);
            self.root_component = self.root_buddy.create_next_menu(replacement_helper);

            // A fresh main component requires a fresh buddy
            self.root_buddy = RootComponentBuddy::new();
            self.root_buddy
                .set_mouse_store(Rc::clone(&self.mouse_store));

            self.root_component.on_attach(&mut self.root_buddy);
            self.work_after_events();
            self.root_buddy.request_render();
        }
    }

    /// Gives the `Application` the opportunity to render its components, or
    /// even `force`s it to do so.
    ///
    /// ### Provider
    /// The *provider* should make sure that this method is called frequently
    /// (typically 60 times per second). If the window resized or lost its
    /// previous pixels, the `force` should be set to true to inform the
    /// application that it should really use this opportunity to render.
    ///
    /// ### Renderer
    /// The application should call the methods of the given `Renderer` to draw itself (if it wants
    /// to draw itself or is `force`d to do so). **Before the application renders anything, it
    /// should call the `start` method of the renderer.** If the application is forced to render,
    /// it is probably wise to call the `clear` method to discard any previous drawing operations
    /// (otherwise, these could remain visible if the root component didn't use all its space).
    /// The methods of the `Renderer` depend on the enabled crate features. In particular, many more
    /// methods will be available if the `golem_rendering` feature is enabled. The core methods will
    /// always be available.
    ///
    /// ### Optional
    /// If the `force` is false, rendering is truly optional: the application can
    /// choose whether or not it wants to redraw itself. To spare power and gpu
    /// time, the application should only do this if something changed. If
    /// nothing changed, the window will keep showing the results of the previous
    /// time the application *did* render.
    ///
    /// ### Return value
    /// This method returns true if the application chose to render (or it was
    /// forced to do so) and false if the application chose not to render.
    pub fn render(&mut self, renderer: &Renderer, force: bool) -> bool {
        if force || self.root_buddy.did_request_render() {
            self.root_buddy.clear_render_request();

            // Make sure we draw onto the right area
            renderer.start();

            // If we are forced to redraw, we should clean the previous render actions up
            if force {
                renderer.clear(Color::rgb(0, 0, 0));
            }

            // Let the root component render itself
            let result = self
                .root_component
                .render(renderer, &mut self.root_buddy, force)
                .expect("Render shouldn't fail");
            self.root_buddy.set_last_render_result(result);

            // Check if the root component requested anything while rendering
            self.work_after_events();
            true
        } else {
            false
        }
    }

    pub fn fire_mouse_click_event(&mut self, event: MouseClickEvent) {
        let sub_mouse_click = self.root_buddy.get_subscriptions().mouse_click;
        let sub_mouse_click_out = self.root_buddy.get_subscriptions().mouse_click_out;

        if sub_mouse_click || sub_mouse_click_out {
            let point = event.get_point();

            let mut fire = false;
            let mut fire_out = false;
            let maybe_render_result = self.root_buddy.get_last_render_result();

            // Don't pass on any click events until the component has been
            // rendered for the first time.
            if let Some(render_result) = maybe_render_result {
                // If we should filter mouse actions, we need to do an additional check
                if render_result.filter_mouse_actions {
                    fire = render_result.drawn_region.is_inside(point);
                } else {
                    fire = true;
                }
                fire_out = !fire;
            }

            if fire {
                self.root_component
                    .on_mouse_click(event, &mut self.root_buddy);
                self.work_after_events();
            }
            if fire_out {
                let out_event = MouseClickOutEvent::new(event.get_mouse(), event.get_button());
                self.root_component
                    .on_mouse_click_out(out_event, &mut self.root_buddy);
                self.work_after_events();
            }
        }
    }

    pub fn fire_mouse_press_event(&mut self, event: MousePressEvent) {
        let mut mouse_store = self.mouse_store.borrow_mut();
        match mouse_store.update_mouse_state(event.get_mouse()) {
            Some(state) => state.buttons.press(event.get_button()),
            None => debug_assert!(false), // Shouldn't happen, but not critical enough for release crash
        };
        drop(mouse_store);

        if self.root_buddy.get_subscriptions().mouse_press {
            if let Some(render_result) = self.root_buddy.get_last_render_result() {
                if !render_result.filter_mouse_actions
                    || render_result.drawn_region.is_inside(event.get_point())
                {
                    self.root_component
                        .on_mouse_press(event, &mut self.root_buddy);
                    self.work_after_events();
                }
            }
        }
    }

    pub fn fire_mouse_release_event(&mut self, event: MouseReleaseEvent) {
        let mut mouse_store = self.mouse_store.borrow_mut();
        match mouse_store.update_mouse_state(event.get_mouse()) {
            Some(state) => state.buttons.release(event.get_button()),
            None => debug_assert!(false), // Shouldn't happen, but not critical enough for release crash
        };
        drop(mouse_store);

        if self.root_buddy.get_subscriptions().mouse_release {
            if let Some(render_result) = self.root_buddy.get_last_render_result() {
                if !render_result.filter_mouse_actions
                    || render_result.drawn_region.is_inside(event.get_point())
                {
                    self.root_component
                        .on_mouse_release(event, &mut self.root_buddy);
                    self.work_after_events();
                }
            }
        }
    }

    fn sub_mouse_enter(&self) -> bool {
        self.root_buddy.get_subscriptions().mouse_enter
    }

    fn sub_mouse_move(&self) -> bool {
        self.root_buddy.get_subscriptions().mouse_move
    }

    fn sub_mouse_leave(&self) -> bool {
        self.root_buddy.get_subscriptions().mouse_leave
    }

    pub fn fire_mouse_move_event(&mut self, event: MouseMoveEvent) {
        // Keep the MouseStore up-to-date
        let mut mouse_store = self.mouse_store.borrow_mut();
        match mouse_store.update_mouse_state(event.get_mouse()) {
            Some(state_to_update) => {
                state_to_update.position = event.get_to();
            }
            None => {
                // This shouldn't happen, but it's not critical enough for a release panic
                debug_assert!(false);
                mouse_store.add_mouse(
                    event.get_mouse(),
                    MouseState {
                        position: event.get_to(),
                        buttons: PressedMouseButtons::new(),
                    },
                );
            }
        };
        drop(mouse_store);

        // Fire the necessary events
        if let Some(render_result) = self.root_buddy.get_last_render_result() {
            // Don't bother doing computations if the root component isn't interested in either event
            if self.sub_mouse_enter() || self.sub_mouse_move() || self.sub_mouse_leave() {
                let filter_mouse = render_result.filter_mouse_actions;
                if filter_mouse {
                    // Complex case: we need to take the render region into account
                    match render_result
                        .drawn_region
                        .find_line_intersection(event.get_from(), event.get_to())
                    {
                        LineIntersection::FullyOutside => {
                            // Do nothing
                        }
                        LineIntersection::FullyInside => {
                            // Simple case: just propagate the event
                            if self.sub_mouse_move() {
                                self.root_component
                                    .on_mouse_move(event, &mut self.root_buddy);
                            }
                        }
                        LineIntersection::Enters { point } => {
                            // Fire a MouseEnterEvent at `point`
                            // and a MouseMoveEvent from `point` to `to`
                            if self.sub_mouse_enter() {
                                let enter_event = MouseEnterEvent::new(event.get_mouse(), point);
                                self.root_component
                                    .on_mouse_enter(enter_event, &mut self.root_buddy);
                            }
                            if self.sub_mouse_move() && event.get_to() != point {
                                let move_event =
                                    MouseMoveEvent::new(event.get_mouse(), point, event.get_to());
                                self.root_component
                                    .on_mouse_move(move_event, &mut self.root_buddy);
                            }
                        }
                        LineIntersection::Exits { point } => {
                            // Fire a MouseMoveEvent from `from` to `point`
                            // and a MouseLeaveEvent at `point`
                            if self.sub_mouse_move() && event.get_from() != point {
                                let move_event =
                                    MouseMoveEvent::new(event.get_mouse(), event.get_from(), point);
                                self.root_component
                                    .on_mouse_move(move_event, &mut self.root_buddy);
                            }
                            if self.sub_mouse_leave() {
                                let leave_event = MouseLeaveEvent::new(event.get_mouse(), point);
                                self.root_component
                                    .on_mouse_leave(leave_event, &mut self.root_buddy);
                            }
                        }
                        LineIntersection::Crosses { entrance, exit } => {
                            // Fire a MouseEnterEvent at `entrance`
                            // and a MouseMoveEvent from `entrance` to `exit`
                            // and a MouseLeaveEvent at `exit`
                            let enter_event = MouseEnterEvent::new(event.get_mouse(), entrance);
                            let move_event = MouseMoveEvent::new(event.get_mouse(), entrance, exit);
                            let leave_event = MouseLeaveEvent::new(event.get_mouse(), exit);
                            if self.sub_mouse_enter() {
                                self.root_component
                                    .on_mouse_enter(enter_event, &mut self.root_buddy);
                            }
                            if self.sub_mouse_move() {
                                self.root_component
                                    .on_mouse_move(move_event, &mut self.root_buddy);
                            }
                            if self.sub_mouse_leave() {
                                self.root_component
                                    .on_mouse_leave(leave_event, &mut self.root_buddy);
                            }
                        }
                    };
                } else {
                    // This is the simple case: just propagate the event
                    if self.sub_mouse_move() {
                        self.root_component
                            .on_mouse_move(event, &mut self.root_buddy);
                    }
                }
                self.work_after_events();
            }
        }
    }

    pub fn fire_mouse_enter_event(&mut self, event: MouseEnterEvent) {
        // Keep the MouseStore up-to-date
        let mut mouse_store = self.mouse_store.borrow_mut();
        mouse_store.add_mouse(
            event.get_mouse(),
            MouseState {
                position: event.get_entrance_point(),
                buttons: PressedMouseButtons::new(),
            },
        );
        drop(mouse_store);

        // Propagate the MouseEnterEvent
        if let Some(render_result) = self.root_buddy.get_last_render_result() {
            if self.root_buddy.get_subscriptions().mouse_enter {
                let should_propagate = match render_result.filter_mouse_actions {
                    true => render_result
                        .drawn_region
                        .is_inside(event.get_entrance_point()),
                    false => true,
                };
                if should_propagate {
                    self.root_component
                        .on_mouse_enter(event, &mut self.root_buddy);
                    self.work_after_events();
                }
            }
        }
    }

    pub fn fire_mouse_leave_event(&mut self, event: MouseLeaveEvent) {
        // Keep the MouseStore up-to-date
        let mut mouse_store = self.mouse_store.borrow_mut();
        mouse_store.remove_mouse(event.get_mouse());
        drop(mouse_store);

        // Propagate the MouseLeaveEvent
        if let Some(render_result) = self.root_buddy.get_last_render_result() {
            if self.root_buddy.get_subscriptions().mouse_leave {
                let should_propagate = match render_result.filter_mouse_actions {
                    true => render_result.drawn_region.is_inside(event.get_exit_point()),
                    false => true,
                };
                if should_propagate {
                    self.root_component
                        .on_mouse_leave(event, &mut self.root_buddy);
                    self.work_after_events();
                }
            }
        }
    }
}

impl Drop for Application {
    fn drop(&mut self) {
        self.root_component.on_detach();
    }
}

#[cfg(test)]
mod tests {

    use crate::*;

    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    struct CountingComponent {
        counter: Rc<Cell<u32>>,
    }

    impl Component for CountingComponent {
        fn on_attach(&mut self, buddy: &mut dyn ComponentBuddy) {
            self.counter.set(self.counter.get() + 1);
            buddy.subscribe_mouse_click();
        }

        fn render(
            &mut self,
            _renderer: &Renderer,
            _buddy: &mut dyn ComponentBuddy,
            _force: bool,
        ) -> RenderResult {
            self.counter.set(self.counter.get() + 3);
            entire_render_result()
        }

        fn on_mouse_click(&mut self, event: MouseClickEvent, buddy: &mut dyn ComponentBuddy) {
            if event.get_point().get_x() > 0.3 {
                buddy.request_render();
            }
            self.counter.set(self.counter.get() + 5);
        }

        fn on_detach(&mut self) {
            self.counter.set(self.counter.get() + 4);
        }
    }

    #[test]
    fn test_initial_attach_and_detach() {
        let counter = Rc::new(Cell::new(0));
        let component = CountingComponent {
            counter: Rc::clone(&counter),
        };
        {
            let _application = Application::new(Box::new(component));

            // The component should have been attached by now
            assert_eq!(1, counter.get());
        }

        // The application (and component) should have been dropped by now
        assert_eq!(1, Rc::strong_count(&counter));
        // And the component should have been detached
        assert_eq!(5, counter.get());
    }

    #[test]
    fn test_render() {
        let counter = Rc::new(Cell::new(0));
        let component = CountingComponent {
            counter: Rc::clone(&counter),
        };
        let mut application = Application::new(Box::new(component));

        let dummy_region = RenderRegion::with_size(0, 0, 150, 100);

        // The component should have been attached, so the counter should be 1
        assert_eq!(1, counter.get());

        // If we simulate 1 render call, the component should draw once
        application.render(&test_renderer(dummy_region), false);
        assert_eq!(4, counter.get());

        // But, rendering again shouldn't change anything because the component
        // didn't request another render
        application.render(&test_renderer(dummy_region), false);
        assert_eq!(4, counter.get());

        // Unless we force it to do so...
        application.render(&test_renderer(dummy_region), true);
        assert_eq!(7, counter.get());

        // After we forced it, things should continue normally...
        application.render(&test_renderer(dummy_region), false);
        assert_eq!(7, counter.get());

        // And no matter how often we request without force, nothing will happen
        for _counter in 0..100 {
            application.render(&test_renderer(dummy_region), false);
            assert_eq!(7, counter.get());
        }
    }

    #[test]
    fn test_click_and_render() {
        let counter = Rc::new(Cell::new(0));
        let component = CountingComponent {
            counter: Rc::clone(&counter),
        };
        let mut application = Application::new(Box::new(component));

        let dummy_region = RenderRegion::between(100, 100, 200, 200);
        let hit_event =
            MouseClickEvent::new(Mouse::new(0), Point::new(0.5, 0.5), MouseButton::primary());
        let miss_event =
            MouseClickEvent::new(Mouse::new(0), Point::new(0.0, 0.0), MouseButton::primary());

        // The counter should be 1 because the component should only have been attached
        assert_eq!(1, counter.get());

        // Rendering 10 times should only increase it once by 3
        for _counter in 0..10 {
            application.render(&test_renderer(dummy_region), false);
        }
        assert_eq!(4, counter.get());

        // If we click (even when we miss), the counter should be increased by 5
        application.fire_mouse_click_event(miss_event);
        assert_eq!(9, counter.get());
        // But rendering won't have effect because we missed
        application.render(&test_renderer(dummy_region), false);
        assert_eq!(9, counter.get());

        // If we hit, the counter should also be increased by 5
        application.fire_mouse_click_event(hit_event);
        assert_eq!(14, counter.get());
        // But this time, rendering will also increase it by 3
        application.render(&test_renderer(dummy_region), false);
        assert_eq!(17, counter.get());

        // But rendering again shouldn't matter
        application.render(&test_renderer(dummy_region), false);
        assert_eq!(17, counter.get());
    }

    #[test]
    fn test_filter_mouse_actions() {
        struct CustomCountingComponent {
            counter: Rc<Cell<u8>>,
            out_counter: Rc<Cell<u8>>,
            press_counter: Rc<Cell<u8>>,
            release_counter: Rc<Cell<u8>>,
        }

        impl Component for CustomCountingComponent {
            fn on_attach(&mut self, buddy: &mut dyn ComponentBuddy) {
                buddy.subscribe_mouse_click();
                buddy.subscribe_mouse_click_out();
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
                    drawn_region: Box::new(RectangularDrawnRegion::new(0.4, 0.4, 0.6, 0.6)),
                    filter_mouse_actions: true,
                })
            }

            fn on_mouse_click(&mut self, _event: MouseClickEvent, _buddy: &mut dyn ComponentBuddy) {
                self.counter.set(self.counter.get() + 1);
            }

            fn on_mouse_click_out(
                &mut self,
                _event: MouseClickOutEvent,
                _buddy: &mut dyn ComponentBuddy,
            ) {
                self.out_counter.set(self.out_counter.get() + 1);
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

        let counter = Rc::new(Cell::new(0));
        let out_counter = Rc::new(Cell::new(0));
        let press_counter = Rc::new(Cell::new(0));
        let release_counter = Rc::new(Cell::new(0));

        let component = CustomCountingComponent {
            counter: Rc::clone(&counter),
            out_counter: Rc::clone(&out_counter),
            press_counter: Rc::clone(&press_counter),
            release_counter: Rc::clone(&release_counter),
        };
        let mut application = Application::new(Box::new(component));

        application
            .fire_mouse_enter_event(MouseEnterEvent::new(Mouse::new(0), Point::new(0.1, 0.1)));
        let miss_click =
            MouseClickEvent::new(Mouse::new(0), Point::new(0.3, 0.3), MouseButton::primary());
        let miss_press =
            MousePressEvent::new(Mouse::new(0), Point::new(0.3, 0.3), MouseButton::primary());
        let miss_release =
            MouseReleaseEvent::new(Mouse::new(0), Point::new(0.3, 0.3), MouseButton::primary());

        let hit_click =
            MouseClickEvent::new(Mouse::new(0), Point::new(0.5, 0.5), MouseButton::primary());
        let hit_press =
            MousePressEvent::new(Mouse::new(0), Point::new(0.5, 0.5), MouseButton::primary());
        let hit_release =
            MouseReleaseEvent::new(Mouse::new(0), Point::new(0.5, 0.5), MouseButton::primary());

        let check_counters = |click: u8, click_out: u8, press: u8, release: u8| {
            assert_eq!(click, counter.get());
            assert_eq!(click_out, out_counter.get());
            assert_eq!(press, press_counter.get());
            assert_eq!(release, release_counter.get());
        };

        // Clicks don't have effect until the component has been drawn
        application.fire_mouse_press_event(hit_press);
        application.fire_mouse_release_event(hit_release);
        application.fire_mouse_click_event(hit_click);
        check_counters(0, 0, 0, 0);

        application.render(&test_renderer(RenderRegion::between(0, 0, 1, 1)), false);

        // Miss clicks should increment only the out counter
        application.fire_mouse_press_event(miss_press);
        application.fire_mouse_release_event(miss_release);
        application.fire_mouse_click_event(miss_click);
        check_counters(0, 1, 0, 0);

        // Hit clicks only increment the real counter
        application.fire_mouse_press_event(hit_press);
        application.fire_mouse_release_event(hit_release);
        application.fire_mouse_click_event(hit_click);
        check_counters(1, 1, 1, 1);
    }

    struct ConditionalMouseFilterComponent {
        should_filter_mouse_actions: Rc<Cell<bool>>,
        mouse_enter_log: Rc<RefCell<Vec<MouseEnterEvent>>>,
        mouse_leave_log: Rc<RefCell<Vec<MouseLeaveEvent>>>,
        mouse_move_log: Rc<RefCell<Vec<MouseMoveEvent>>>,
    }

    impl Component for ConditionalMouseFilterComponent {
        fn on_attach(&mut self, buddy: &mut dyn ComponentBuddy) {
            buddy.subscribe_mouse_enter();
            buddy.subscribe_mouse_move();
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
                drawn_region: Box::new(RectangularDrawnRegion::new(0.2, 0.0, 0.8, 0.5)),
            })
        }

        fn on_mouse_move(&mut self, event: MouseMoveEvent, _buddy: &mut dyn ComponentBuddy) {
            let mut move_events = self.mouse_move_log.borrow_mut();
            move_events.push(event);
        }

        fn on_mouse_enter(&mut self, event: MouseEnterEvent, _buddy: &mut dyn ComponentBuddy) {
            let mut enter_events = self.mouse_enter_log.borrow_mut();
            enter_events.push(event);
        }

        fn on_mouse_leave(&mut self, event: MouseLeaveEvent, _buddy: &mut dyn ComponentBuddy) {
            let mut leave_events = self.mouse_leave_log.borrow_mut();
            leave_events.push(event);
        }
    }

    #[test]
    fn test_mouse_enter_and_leave() {
        let should_filter_mouse_actions = Rc::new(Cell::new(false));
        let mouse_enter_log = Rc::new(RefCell::new(Vec::new()));
        let mouse_leave_log = Rc::new(RefCell::new(Vec::new()));

        let component = ConditionalMouseFilterComponent {
            should_filter_mouse_actions: Rc::clone(&should_filter_mouse_actions),
            mouse_move_log: Rc::new(RefCell::new(Vec::new())),
            mouse_enter_log: Rc::clone(&mouse_enter_log),
            mouse_leave_log: Rc::clone(&mouse_leave_log),
        };

        let mut application = Application::new(Box::new(component));

        let outer_enter_event = MouseEnterEvent::new(Mouse::new(0), Point::new(0.1, 0.1));
        let outer_leave_event = MouseLeaveEvent::new(Mouse::new(0), Point::new(0.1, 0.1));
        let inner_enter_event = MouseEnterEvent::new(Mouse::new(0), Point::new(0.4, 0.4));
        let inner_leave_event = MouseLeaveEvent::new(Mouse::new(0), Point::new(0.4, 0.4));
        let render_region = RenderRegion::between(12, 123, 1234, 12345);

        let check_enters = |expected: Vec<MouseEnterEvent>| {
            let enter_log = mouse_enter_log.borrow();
            assert_eq!(expected, *enter_log);
        };
        let check_leaves = |expected: Vec<MouseLeaveEvent>| {
            let leave_log = mouse_leave_log.borrow();
            assert_eq!(expected, *leave_log);
        };

        check_enters(vec![]);
        check_leaves(vec![]);

        // These events should be ignored until the component has been rendered for the first time
        application.fire_mouse_enter_event(inner_enter_event);
        application.fire_mouse_leave_event(inner_leave_event);
        check_enters(vec![]);
        check_leaves(vec![]);

        // But events after the first render should be registered
        application.render(&test_renderer(render_region), false);
        check_enters(vec![]);
        check_leaves(vec![]);
        application.fire_mouse_enter_event(inner_enter_event);
        application.fire_mouse_leave_event(inner_leave_event);
        application.fire_mouse_enter_event(outer_enter_event);
        application.fire_mouse_leave_event(outer_leave_event);
        check_enters(vec![inner_enter_event, outer_enter_event]);
        check_leaves(vec![inner_leave_event, outer_leave_event]);

        // If we enable mouse filtering, only the inner events should be received
        should_filter_mouse_actions.set(true);
        application.render(&test_renderer(render_region), true);
        application.fire_mouse_enter_event(inner_enter_event);
        application.fire_mouse_leave_event(inner_leave_event);
        application.fire_mouse_enter_event(outer_enter_event);
        application.fire_mouse_leave_event(outer_leave_event);
        check_enters(vec![
            inner_enter_event,
            outer_enter_event,
            inner_enter_event,
        ]);
        check_leaves(vec![
            inner_leave_event,
            outer_leave_event,
            inner_leave_event,
        ]);
    }

    #[test]
    fn test_mouse_move_subscriptions() {
        let should_filter_mouse_actions = Rc::new(Cell::new(false));
        let received_mouse_move = Rc::new(Cell::new(false));
        let received_mouse_enter = Rc::new(Cell::new(false));
        let received_mouse_leave = Rc::new(Cell::new(false));

        let check_received = |did_move: bool, did_enter: bool, did_leave: bool| {
            assert_eq!(did_move, received_mouse_move.get());
            assert_eq!(did_enter, received_mouse_enter.get());
            assert_eq!(did_leave, received_mouse_leave.get());
            received_mouse_move.set(false);
            received_mouse_enter.set(false);
            received_mouse_leave.set(false);
        };

        struct SubscriptionState {
            mouse_move: bool,
            mouse_enter: bool,
            mouse_leave: bool,
        }

        let control_subscriptions = Rc::new(RefCell::new(SubscriptionState {
            mouse_move: false,
            mouse_enter: false,
            mouse_leave: false,
        }));

        let set_subscriptions =
            |sub_mouse_move: bool, sub_mouse_enter: bool, sub_mouse_leave: bool| {
                let mut subscriptions = control_subscriptions.borrow_mut();
                subscriptions.mouse_move = sub_mouse_move;
                subscriptions.mouse_enter = sub_mouse_enter;
                subscriptions.mouse_leave = sub_mouse_leave;
            };

        struct SubscriptionComponent {
            should_filter_mouse_actions: Rc<Cell<bool>>,
            subscriptions: Rc<RefCell<SubscriptionState>>,

            received_mouse_move: Rc<Cell<bool>>,
            received_mouse_enter: Rc<Cell<bool>>,
            received_mouse_leave: Rc<Cell<bool>>,
        }

        impl Component for SubscriptionComponent {
            fn on_attach(&mut self, _buddy: &mut dyn ComponentBuddy) {}

            fn render(
                &mut self,
                _renderer: &Renderer,
                buddy: &mut dyn ComponentBuddy,
                _force: bool,
            ) -> RenderResult {
                let subscriptions = self.subscriptions.borrow();
                if subscriptions.mouse_move {
                    buddy.subscribe_mouse_move();
                } else {
                    buddy.unsubscribe_mouse_move();
                }
                if subscriptions.mouse_enter {
                    buddy.subscribe_mouse_enter();
                } else {
                    buddy.unsubscribe_mouse_enter();
                }
                if subscriptions.mouse_leave {
                    buddy.subscribe_mouse_leave();
                } else {
                    buddy.unsubscribe_mouse_leave();
                }
                Ok(RenderResultStruct {
                    filter_mouse_actions: self.should_filter_mouse_actions.get(),
                    drawn_region: Box::new(RectangularDrawnRegion::new(0.2, 0.2, 0.6, 0.6)),
                })
            }

            fn on_mouse_move(&mut self, _event: MouseMoveEvent, _buddy: &mut dyn ComponentBuddy) {
                self.received_mouse_move.set(true);
            }

            fn on_mouse_enter(&mut self, _event: MouseEnterEvent, _buddy: &mut dyn ComponentBuddy) {
                self.received_mouse_enter.set(true);
            }

            fn on_mouse_leave(&mut self, _event: MouseLeaveEvent, _buddy: &mut dyn ComponentBuddy) {
                self.received_mouse_leave.set(true);
            }
        }

        let component = SubscriptionComponent {
            should_filter_mouse_actions: Rc::clone(&should_filter_mouse_actions),
            subscriptions: Rc::clone(&control_subscriptions),
            received_mouse_move: Rc::clone(&received_mouse_move),
            received_mouse_enter: Rc::clone(&received_mouse_enter),
            received_mouse_leave: Rc::clone(&received_mouse_leave),
        };

        let mut application = Application::new(Box::new(component));
        application
            .fire_mouse_enter_event(MouseEnterEvent::new(Mouse::new(0), Point::new(0.0, 0.4)));
        let the_event =
            MouseMoveEvent::new(Mouse::new(0), Point::new(0.0, 0.4), Point::new(1.0, 0.4));
        let render_region = RenderRegion::with_size(0, 0, 30, 70);

        // It shouldn't have subscribed to any of the events yet
        application.render(&test_renderer(render_region), false);
        application.fire_mouse_move_event(the_event);
        check_received(false, false, false);

        // Until we filter mouse events, only mouse move can be received
        set_subscriptions(true, true, true);
        application.render(&test_renderer(render_region), true);
        application.fire_mouse_move_event(the_event);
        check_received(true, false, false);

        // But things get more complex when we do filter mouse events
        let mut test_combination = |mouse_move: bool, mouse_enter: bool, mouse_leave: bool| {
            set_subscriptions(mouse_move, mouse_enter, mouse_leave);
            application.render(&test_renderer(render_region), true);
            application.fire_mouse_move_event(the_event);
            check_received(mouse_move, mouse_enter, mouse_leave);
        };
        should_filter_mouse_actions.set(true);

        // Try all 8 combinations twice
        for _counter in 0..2 {
            test_combination(false, false, true);
            test_combination(false, true, false);
            test_combination(false, true, true);
            test_combination(true, false, false);
            test_combination(true, false, true);
            test_combination(true, true, false);
            test_combination(true, true, true);
        }
    }

    #[test]
    fn test_mouse_move() {
        let mouse_move_log = Rc::new(RefCell::new(Vec::new()));
        let mouse_enter_log = Rc::new(RefCell::new(Vec::new()));
        let mouse_leave_log = Rc::new(RefCell::new(Vec::new()));

        let check_counts = |mouse_move: usize, mouse_enter: usize, mouse_leave: usize| {
            let mouse_moves = mouse_move_log.borrow();
            assert_eq!(mouse_move, mouse_moves.len());
            let mouse_enters = mouse_enter_log.borrow();
            assert_eq!(mouse_enter, mouse_enters.len());
            let mouse_leaves = mouse_leave_log.borrow();
            assert_eq!(mouse_leave, mouse_leaves.len());
        };

        let component = ConditionalMouseFilterComponent {
            should_filter_mouse_actions: Rc::new(Cell::new(true)),
            mouse_move_log: Rc::clone(&mouse_move_log),
            mouse_enter_log: Rc::clone(&mouse_enter_log),
            mouse_leave_log: Rc::clone(&mouse_leave_log),
        };

        let mut application = Application::new(Box::new(component));
        application.render(&test_renderer(RenderRegion::between(1, 2, 3, 4)), false);

        // Let the mouse enter the application
        application
            .fire_mouse_enter_event(MouseEnterEvent::new(Mouse::new(0), Point::new(0.0, 1.0)));

        // Move the mouse entirely outside
        let outside_event =
            MouseMoveEvent::new(Mouse::new(0), Point::new(0.0, 1.0), Point::new(0.0, 0.0));
        application.fire_mouse_move_event(outside_event);
        check_counts(0, 0, 0);

        // Move the mouse from outside to inside the component
        let enter_event =
            MouseMoveEvent::new(Mouse::new(0), Point::new(0.0, 0.0), Point::new(0.4, 0.2));
        application.fire_mouse_move_event(enter_event);
        check_counts(1, 1, 0);

        // Move the mouse entirely inside the component
        let inside_event =
            MouseMoveEvent::new(Mouse::new(0), Point::new(0.4, 0.2), Point::new(0.6, 0.3));
        application.fire_mouse_move_event(inside_event);
        check_counts(2, 1, 0);

        // Move the mouse from inside to outside the component
        let leave_event =
            MouseMoveEvent::new(Mouse::new(0), Point::new(0.6, 0.3), Point::new(1.0, 0.5));
        application.fire_mouse_move_event(leave_event);
        check_counts(3, 1, 1);

        // We already checked that the number of events fired is correct, but we haven't checked the
        // parameters of the events yet.
        let move_events = mouse_move_log.borrow();
        assert!(move_events[0].get_from().nearly_equal(Point::new(0.2, 0.1)));
        assert!(move_events[0].get_to().nearly_equal(Point::new(0.4, 0.2)));
        assert!(move_events[1].get_from().nearly_equal(Point::new(0.4, 0.2)));
        assert!(move_events[1].get_to().nearly_equal(Point::new(0.6, 0.3)));
        assert!(move_events[2].get_from().nearly_equal(Point::new(0.6, 0.3)));
        assert!(move_events[2].get_to().nearly_equal(Point::new(0.8, 0.4)));

        let enter_events = mouse_enter_log.borrow();
        assert!(enter_events[0]
            .get_entrance_point()
            .nearly_equal(Point::new(0.2, 0.1)));

        let leave_events = mouse_leave_log.borrow();
        assert!(leave_events[0]
            .get_exit_point()
            .nearly_equal(Point::new(0.8, 0.4)));
    }

    #[test]
    fn test_subscribe_and_unsubscribe() {
        struct EventFlags {
            mouse_click: bool,
            mouse_press: bool,
            mouse_release: bool,
            mouse_enter: bool,
            mouse_leave: bool,
        }

        struct SubscribeComponent {
            desired_subscriptions: Rc<RefCell<EventFlags>>,
            received_events: Rc<RefCell<EventFlags>>,
        }

        impl Component for SubscribeComponent {
            fn on_attach(&mut self, _buddy: &mut dyn ComponentBuddy) {}

            fn render(
                &mut self,
                _renderer: &Renderer,
                buddy: &mut dyn ComponentBuddy,
                _force: bool,
            ) -> RenderResult {
                let new_subscriptions = self.desired_subscriptions.borrow();
                if new_subscriptions.mouse_click {
                    buddy.subscribe_mouse_click();
                } else {
                    buddy.unsubscribe_mouse_click();
                }
                if new_subscriptions.mouse_press {
                    buddy.subscribe_mouse_press();
                } else {
                    buddy.unsubscribe_mouse_press();
                }
                if new_subscriptions.mouse_release {
                    buddy.subscribe_mouse_release();
                } else {
                    buddy.unsubscribe_mouse_release();
                }
                if new_subscriptions.mouse_enter {
                    buddy.subscribe_mouse_enter();
                } else {
                    buddy.unsubscribe_mouse_enter();
                }
                if new_subscriptions.mouse_leave {
                    buddy.subscribe_mouse_leave();
                } else {
                    buddy.unsubscribe_mouse_leave();
                }
                entire_render_result()
            }

            fn on_mouse_click(&mut self, _event: MouseClickEvent, _buddy: &mut dyn ComponentBuddy) {
                let mut flags = self.received_events.borrow_mut();
                flags.mouse_click = true;
            }

            fn on_mouse_press(&mut self, _event: MousePressEvent, _buddy: &mut dyn ComponentBuddy) {
                let mut flags = self.received_events.borrow_mut();
                flags.mouse_press = true;
            }

            fn on_mouse_release(
                &mut self,
                _event: MouseReleaseEvent,
                _buddy: &mut dyn ComponentBuddy,
            ) {
                let mut flags = self.received_events.borrow_mut();
                flags.mouse_release = true;
            }

            fn on_mouse_enter(&mut self, _event: MouseEnterEvent, _buddy: &mut dyn ComponentBuddy) {
                let mut flags = self.received_events.borrow_mut();
                flags.mouse_enter = true;
            }

            fn on_mouse_leave(&mut self, _event: MouseLeaveEvent, _buddy: &mut dyn ComponentBuddy) {
                let mut flags = self.received_events.borrow_mut();
                flags.mouse_leave = true;
            }
        }

        let desired_subscriptions = Rc::new(RefCell::new(EventFlags {
            mouse_click: false,
            mouse_press: false,
            mouse_release: false,
            mouse_enter: false,
            mouse_leave: false,
        }));
        let received_events = Rc::new(RefCell::new(EventFlags {
            mouse_click: false,
            mouse_press: false,
            mouse_release: false,
            mouse_enter: false,
            mouse_leave: false,
        }));

        let component = SubscribeComponent {
            desired_subscriptions: Rc::clone(&desired_subscriptions),
            received_events: Rc::clone(&received_events),
        };

        let mut application = Application::new(Box::new(component));
        let mut try_events = |mouse_click: bool,
                              mouse_press: bool,
                              mouse_release: bool,
                              mouse_enter: bool,
                              mouse_leave: bool| {
            let mut subscribe = desired_subscriptions.borrow_mut();
            subscribe.mouse_click = mouse_click;
            subscribe.mouse_press = mouse_press;
            subscribe.mouse_release = mouse_release;
            subscribe.mouse_enter = mouse_enter;
            subscribe.mouse_leave = mouse_leave;
            drop(subscribe);

            let mut clear_received_flags = received_events.borrow_mut();
            clear_received_flags.mouse_click = false;
            clear_received_flags.mouse_press = false;
            clear_received_flags.mouse_release = false;
            clear_received_flags.mouse_enter = false;
            clear_received_flags.mouse_leave = false;
            drop(clear_received_flags);

            let render_region = RenderRegion::between(1, 2, 3, 4);
            application.render(&test_renderer(render_region), true);

            let point = Point::new(0.5, 0.5);
            let mouse = Mouse::new(0);
            let button = MouseButton::primary();
            let enter_event = MouseEnterEvent::new(mouse, point);
            let press_event = MousePressEvent::new(mouse, point, button);
            let release_event = MouseReleaseEvent::new(mouse, point, button);
            let click_event = MouseClickEvent::new(mouse, point, button);
            let leave_event = MouseLeaveEvent::new(mouse, point);

            application.fire_mouse_enter_event(enter_event);
            application.fire_mouse_press_event(press_event);
            application.fire_mouse_release_event(release_event);
            application.fire_mouse_click_event(click_event);
            application.fire_mouse_leave_event(leave_event);

            let check_received_flags = received_events.borrow_mut();
            assert_eq!(mouse_click, check_received_flags.mouse_click);
            assert_eq!(mouse_press, check_received_flags.mouse_press);
            assert_eq!(mouse_release, check_received_flags.mouse_release);
            assert_eq!(mouse_enter, check_received_flags.mouse_enter);
            assert_eq!(mouse_leave, check_received_flags.mouse_leave);
        };

        // Try every combination of subscriptions, and do it twice to test even more
        for _counter in 0..2 {
            for binary_int in 0..32 {
                let b1 = binary_int & 1 != 0;
                let b2 = binary_int & 2 != 0;
                let b3 = binary_int & 4 != 0;
                let b4 = binary_int & 8 != 0;
                let b5 = binary_int & 16 != 0;
                try_events(b1, b2, b3, b4, b5);
            }
        }
    }

    #[test]
    fn test_buddy_get_mouses() {
        struct GetMouseComponent {
            expected: Rc<RefCell<Vec<Mouse>>>,
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
                assert_eq!(expected.as_ref() as &Vec<Mouse>, &buddy.get_local_mouses());
                assert_eq!(expected.as_ref() as &Vec<Mouse>, &buddy.get_all_mouses());
                entire_render_result()
            }
        }

        let expected_mouses = Rc::new(RefCell::new(Vec::new()));

        let mut application = Application::new(Box::new(GetMouseComponent {
            expected: Rc::clone(&expected_mouses),
        }));

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
    }

    #[test]
    fn test_buddy_get_mouse_position() {
        #[derive(Copy, Clone)]
        struct MouseCheck {
            mouse: Mouse,
            expected_position: Option<Point>,
        }

        fn check(mouse: Mouse, expected_x: f32, expected_y: f32) -> MouseCheck {
            MouseCheck {
                mouse,
                expected_position: Some(Point::new(expected_x, expected_y)),
            }
        }

        fn check_none(mouse: Mouse) -> MouseCheck {
            MouseCheck {
                mouse,
                expected_position: None,
            }
        }

        struct MouseCheckingComponent {
            check: Rc<Cell<MouseCheck>>,
        }

        impl Component for MouseCheckingComponent {
            fn on_attach(&mut self, _buddy: &mut dyn ComponentBuddy) {}

            fn render(
                &mut self,
                _renderer: &Renderer,
                buddy: &mut dyn ComponentBuddy,
                _force: bool,
            ) -> RenderResult {
                assert_eq!(
                    self.check.get().expected_position,
                    buddy.get_mouse_position(self.check.get().mouse)
                );
                entire_render_result()
            }
        }

        let mouse1 = Mouse::new(1);
        let mouse2 = Mouse::new(0);

        let next_check = Rc::new(Cell::new(check_none(mouse1)));

        let region = RenderRegion::with_size(1, 2, 3, 4);
        let mut application = Application::new(Box::new(MouseCheckingComponent {
            check: Rc::clone(&next_check),
        }));
        application.render(&test_renderer(region), true);
        application.fire_mouse_enter_event(MouseEnterEvent::new(mouse1, Point::new(0.3, 0.4)));
        next_check.set(check(mouse1, 0.3, 0.4));
        application.render(&test_renderer(region), true);
        next_check.set(check_none(mouse2));
        application.render(&test_renderer(region), true);
        application.fire_mouse_move_event(MouseMoveEvent::new(
            mouse1,
            Point::new(0.3, 0.4),
            Point::new(0.6, 0.5),
        ));
        next_check.set(check(mouse1, 0.6, 0.5));
        application.render(&test_renderer(region), true);

        application.fire_mouse_enter_event(MouseEnterEvent::new(mouse2, Point::new(0.1, 0.2)));
        next_check.set(check(mouse2, 0.1, 0.2));
        application.render(&test_renderer(region), true);
        next_check.set(check(mouse1, 0.6, 0.5));
        application.render(&test_renderer(region), true);

        application.fire_mouse_move_event(MouseMoveEvent::new(
            mouse2,
            Point::new(0.1, 0.2),
            Point::new(0.7, 0.1),
        ));
        application.render(&test_renderer(region), true);
        next_check.set(check(mouse2, 0.7, 0.1));
        application.render(&test_renderer(region), true);

        application.fire_mouse_leave_event(MouseLeaveEvent::new(mouse1, Point::new(0.6, 0.5)));
        next_check.set(check_none(mouse1));
        application.render(&test_renderer(region), true);
        next_check.set(check(mouse2, 0.7, 0.1));
        application.render(&test_renderer(region), true);
    }

    #[test]
    fn test_buddy_pressed_mouse_buttons() {
        #[derive(Copy, Clone)]
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

        struct MouseCheckComponent {
            next_check: Rc<Cell<MouseCheck>>,
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
                self.render_counter.set(self.render_counter.get() + 1);
                let next_check = self.next_check.get();
                assert_eq!(
                    next_check.result,
                    buddy.is_mouse_button_down(next_check.mouse, next_check.button)
                );

                Ok(RenderResultStruct {
                    filter_mouse_actions: true,
                    drawn_region: Box::new(RectangularDrawnRegion::new(0.2, 0.2, 0.8, 0.8)),
                })
            }
        }

        let mouse1 = Mouse::new(1);
        let mouse2 = Mouse::new(2);

        let button = MouseButton::primary();

        let render_counter = Rc::new(Cell::new(0));
        let next_check = Rc::new(Cell::new(MouseCheck::new(mouse1, button, None)));

        let mut application = Application::new(Box::new(MouseCheckComponent {
            render_counter: Rc::clone(&render_counter),
            next_check: Rc::clone(&next_check),
        }));
        let renderer = test_renderer(RenderRegion::with_size(0, 0, 10, 20));

        application.render(&renderer, false);
        assert_eq!(1, render_counter.get());

        let miss_point = Point::new(0.1, 0.1);
        let middle = Point::new(0.5, 0.5);

        application.fire_mouse_enter_event(MouseEnterEvent::new(mouse1, miss_point));
        application.fire_mouse_press_event(MousePressEvent::new(mouse1, miss_point, button));

        // The component filters mouse actions and this is outside its drawn region
        application.render(&renderer, true);
        assert_eq!(2, render_counter.get());

        // But if we move the mouse inside...
        application.fire_mouse_move_event(MouseMoveEvent::new(mouse1, miss_point, middle));
        next_check.set(MouseCheck::new(mouse1, button, Some(true)));
        application.render(&renderer, true);
        assert_eq!(3, render_counter.get());

        // Let's also add the other mouse
        application.fire_mouse_enter_event(MouseEnterEvent::new(mouse2, middle));
        application.fire_mouse_press_event(MousePressEvent::new(mouse2, middle, button));
        next_check.set(MouseCheck::new(mouse2, button, Some(true)));
        application.render(&renderer, true);
        assert_eq!(4, render_counter.get());

        // Let's release mouse 1
        application.fire_mouse_release_event(MouseReleaseEvent::new(mouse1, middle, button));
        next_check.set(MouseCheck::new(mouse1, button, Some(false)));
        application.render(&renderer, true);
        assert_eq!(5, render_counter.get());
        // Mouse 2 should still be pressed
        next_check.set(MouseCheck::new(mouse2, button, Some(true)));

        // Let's move mouse 2 away
        application.fire_mouse_move_event(MouseMoveEvent::new(mouse2, middle, miss_point));
        next_check.set(MouseCheck::new(mouse2, button, None));
        application.render(&renderer, true);
        assert_eq!(6, render_counter.get());
        // Mouse 1 should still be released
        next_check.set(MouseCheck::new(mouse1, button, Some(false)));
        application.render(&renderer, true);
        assert_eq!(7, render_counter.get());

        // Let mouse 1 leave
        application.fire_mouse_leave_event(MouseLeaveEvent::new(mouse1, middle));
        next_check.set(MouseCheck::new(mouse1, button, None));
        application.render(&renderer, true);
        assert_eq!(8, render_counter.get());
    }

    #[test]
    fn test_change_menu() {
        struct ChangingComponent {
            click_counter: Rc<Cell<u8>>,
            changed_counter: Rc<Cell<u8>>,
        }

        impl Component for ChangingComponent {
            fn on_attach(&mut self, buddy: &mut dyn ComponentBuddy) {
                buddy.subscribe_mouse_click();
            }

            fn render(
                &mut self,
                _renderer: &Renderer,
                _buddy: &mut dyn ComponentBuddy,
                _force: bool,
            ) -> RenderResult {
                entire_render_result()
            }

            fn on_mouse_click(&mut self, _event: MouseClickEvent, buddy: &mut dyn ComponentBuddy) {
                self.click_counter.set(self.click_counter.get() + 1);
                let changed_counter = Rc::clone(&self.changed_counter);
                buddy.change_menu(Box::new(move |_old_menu: Box<dyn Component>| {
                    Box::new(ChangedComponent {
                        counter: changed_counter,
                    })
                }));
            }

            fn on_detach(&mut self) {
                self.click_counter.set(self.click_counter.get() * 4);
            }
        }

        struct ChangedComponent {
            counter: Rc<Cell<u8>>,
        }

        impl Component for ChangedComponent {
            fn on_attach(&mut self, buddy: &mut dyn ComponentBuddy) {
                buddy.subscribe_mouse_click();
                self.counter.set(10);
            }

            fn render(
                &mut self,
                _renderer: &Renderer,
                _buddy: &mut dyn ComponentBuddy,
                _force: bool,
            ) -> RenderResult {
                self.counter.set(self.counter.get() * 5);
                entire_render_result()
            }

            fn on_mouse_click(&mut self, _event: MouseClickEvent, _buddy: &mut dyn ComponentBuddy) {
                self.counter.set(self.counter.get() + 1);
            }
        }

        let counter1 = Rc::new(Cell::new(0));
        let counter2 = Rc::new(Cell::new(0));

        let mut application = Application::new(Box::new(ChangingComponent {
            click_counter: Rc::clone(&counter1),
            changed_counter: Rc::clone(&counter2),
        }));

        let click_event =
            MouseClickEvent::new(Mouse::new(0), Point::new(0.2, 0.6), MouseButton::primary());
        let renderer = test_renderer(RenderRegion::with_size(1, 2, 3, 4));

        application.render(&renderer, false);
        // Firing the click event should cause the second component to be attached
        application.fire_mouse_click_event(click_event);
        assert_eq!(4, counter1.get());
        assert_eq!(10, counter2.get());

        // It should receive the render event
        application.render(&renderer, false);
        assert_eq!(50, counter2.get());

        // And this click event
        application.fire_mouse_click_event(click_event);
        assert_eq!(51, counter2.get());

        // Rendering it again shouldn't have any effect
        application.render(&renderer, false);
        assert_eq!(51, counter2.get());

        // And component 1 shouldn't have received any more events
        assert_eq!(4, counter1.get());
    }
}
