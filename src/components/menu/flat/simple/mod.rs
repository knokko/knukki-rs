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
}

impl SimpleFlatMenu {
    pub fn new(background_color: Option<Color>) -> Self {
        Self {
            components: Vec::new(),
            components_to_add: Vec::new(),
            background_color,
            has_rendered_before: false,
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
                buddy: SimpleFlatBuddy::new(),
            };

            entry_to_add.attach();
            self.check_buddy(own_buddy, &mut entry_to_add, is_about_to_render);

            // Don't forget this x)
            self.components.push(Rc::new(RefCell::new(entry_to_add)));
        }
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
        // TODO Performance improvement: only subscribe when at least 1 component did
        buddy.subscribe_mouse_click();
        buddy.subscribe_mouse_click_out();
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
                // TODO This really needs a scissor...
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

        let mut buddy = RootComponentBuddy::new();
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
        let mut buddy = RootComponentBuddy::new();
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
        let mut buddy = RootComponentBuddy::new();

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

        let mut buddy = RootComponentBuddy::new();
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

        let buddy = RootComponentBuddy::new();
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
}
