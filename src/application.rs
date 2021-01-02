use crate::*;

#[cfg(feature = "golem_rendering")]
use golem::Context;

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
}

impl Application {
    pub fn new(mut initial_root_component: Box<dyn Component>) -> Self {
        let mut root_buddy = RootComponentBuddy::new();
        initial_root_component.on_attach(&mut root_buddy);
        // No need to call request_render, because the did_request_render field
        // of RootComponentBuddy starts as true
        let mut result = Self {
            root_component: initial_root_component,
            root_buddy,
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
    /// ### Golem context
    /// When the `golem_rendering` feature is enabled, this method expects
    /// a Golem `Context` as first parameter. This is the context where
    /// the application will render its components. If this feature is not
    /// enabled, the application will perform a 'dummy render': The
    /// components will 'pretend' that they are drawing itself and should
    /// return the same `RenderResult` as they would when given an actual
    /// golem `Context`. This is of course useless for production environments
    /// because the application will be invisible, but very useful for unit
    /// testing: there is no need to create some dirty offscreen window that
    /// nobody will be able to view anyway.
    ///
    /// ### Region
    /// The *provider* can use the *region* parameter to tell the application
    /// where it should render itself within the given golem `Context`. This
    /// should normally cover the entire inner window, but the provider is
    /// allowed to choose a different region.
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
    pub fn render(
        &mut self,
        #[cfg(feature = "golem_rendering")] golem: &Context,
        region: RenderRegion,
        force: bool,
    ) -> bool {
        if force || self.root_buddy.did_request_render() {
            self.root_buddy.clear_render_request();

            // Make sure we draw onto the right area
            #[cfg(feature = "golem_rendering")]
            region.set_viewport(golem);

            // Let the root component render itself
            let result = self
                .root_component
                .render(
                    #[cfg(feature = "golem_rendering")]
                    golem,
                    region,
                    &mut self.root_buddy,
                    force,
                )
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
            _region: RenderRegion,
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
        application.render(dummy_region, false);
        assert_eq!(4, counter.get());

        // But, rendering again shouldn't change anything because the component
        // didn't request another render
        application.render(dummy_region, false);
        assert_eq!(4, counter.get());

        // Unless we force it to do so...
        application.render(dummy_region, true);
        assert_eq!(7, counter.get());

        // After we forced it, things should continue normally...
        application.render(dummy_region, false);
        assert_eq!(7, counter.get());

        // And no matter how often we request without force, nothing will happen
        for _counter in 0..100 {
            application.render(dummy_region, false);
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
            application.render(dummy_region, false);
        }
        assert_eq!(4, counter.get());

        // If we click (even when we miss), the counter should be increased by 5
        application.fire_mouse_click_event(miss_event);
        assert_eq!(9, counter.get());
        // But rendering won't have effect because we missed
        application.render(dummy_region, false);
        assert_eq!(9, counter.get());

        // If we hit, the counter should also be increased by 5
        application.fire_mouse_click_event(hit_event);
        assert_eq!(14, counter.get());
        // But this time, rendering will also increase it by 3
        application.render(dummy_region, false);
        assert_eq!(17, counter.get());

        // But rendering again shouldn't matter
        application.render(dummy_region, false);
        assert_eq!(17, counter.get());
    }

    #[test]
    fn test_filter_mouse_actions() {
        struct CustomCountingComponent {
            counter: Rc<Cell<u8>>,
            out_counter: Rc<Cell<u8>>,
        }

        impl Component for CustomCountingComponent {
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
        }
        let counter = Rc::new(Cell::new(0));
        let out_counter = Rc::new(Cell::new(0));
        let component = CustomCountingComponent {
            counter: Rc::clone(&counter),
            out_counter: Rc::clone(&out_counter),
        };
        let mut application = Application::new(Box::new(component));

        let miss_click =
            MouseClickEvent::new(Mouse::new(0), Point::new(0.3, 0.3), MouseButton::primary());
        let hit_click =
            MouseClickEvent::new(Mouse::new(0), Point::new(0.5, 0.5), MouseButton::primary());

        // Clicks don't have effect until the component has been drawn
        application.fire_mouse_click_event(hit_click);
        assert_eq!(0, counter.get());
        assert_eq!(0, out_counter.get());

        application.render(RenderRegion::between(0, 0, 1, 1), false);

        // Miss clicks should increment only the out counter
        application.fire_mouse_click_event(miss_click);
        assert_eq!(0, counter.get());
        assert_eq!(1, out_counter.get());

        // Hit clicks only increment the real counter
        application.fire_mouse_click_event(hit_click);
        assert_eq!(1, counter.get());
        assert_eq!(1, out_counter.get());
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
            _region: RenderRegion,
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
        application.render(render_region, false);
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
        application.render(render_region, true);
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
                _region: RenderRegion,
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
        let the_event =
            MouseMoveEvent::new(Mouse::new(0), Point::new(0.0, 0.4), Point::new(1.0, 0.4));
        let render_region = RenderRegion::with_size(0, 0, 30, 70);

        // It shouldn't have subscribed to any of the events yet
        application.render(render_region, false);
        application.fire_mouse_move_event(the_event);
        check_received(false, false, false);

        // Until we filter mouse events, only mouse move can be received
        set_subscriptions(true, true, true);
        application.render(render_region, true);
        application.fire_mouse_move_event(the_event);
        check_received(true, false, false);

        // But things get more complex when we do filter mouse events
        let mut test_combination = |mouse_move: bool, mouse_enter: bool, mouse_leave: bool| {
            set_subscriptions(mouse_move, mouse_enter, mouse_leave);
            application.render(render_region, true);
            application.fire_mouse_move_event(the_event);
            check_received(mouse_move, mouse_enter, mouse_leave);
        };
        should_filter_mouse_actions.set(true);

        // Try all 8 combinations
        test_combination(false, false, true);
        test_combination(false, true, false);
        test_combination(false, true, true);
        test_combination(true, false, false);
        test_combination(true, false, true);
        test_combination(true, true, false);
        test_combination(true, true, true);

        // Also try a full unsubscribe
        test_combination(false, false, false);
    }

    // TODO Test mouse move subscriptions
    // TODO Test mouse move in general
    // TODO Test general subscriptions and unsubscriptions for all events
}
