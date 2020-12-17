use crate::*;
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
        root_buddy.request_render();
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
    pub fn render(&mut self, golem: &Context, region: RenderRegion, force: bool) {
        if force || self.root_buddy.did_request_render() {
            self.root_buddy.clear_render_request();

            // Make sure we draw onto the right area
            region.set_viewport(golem);

            // Let the root component render itself
            let result = self
                .root_component
                .render(golem, region, &mut self.root_buddy);
            self.root_buddy.set_last_render_result(result);

            // Check if the root component requested anything while rendering
            self.work_after_events();
        }
    }

    /// Let the `Application` pretend like it received a `render` call. But
    /// unlike a real `render` call, nothing will be rendered.
    /// 
    /// ### Purpose
    /// The purpose of this method is to make unit tests easier: no real
    /// Golem rendering context is necessary. If the components of the
    /// application use reasonable implementations of `simulate_render`,
    /// the testing will still be very accurate.
    pub fn simulate_render(&mut self, region: RenderRegion, force: bool) {
        if force || self.root_buddy.did_request_render() {
            self.root_buddy.clear_render_request();

            let result = self.root_component.simulate_render(region, &mut self.root_buddy);
            self.root_buddy.set_last_render_result(result);

            self.work_after_events();
        }
    }

    pub fn fire_mouse_click_event(&mut self, event: MouseClickEvent) {
        if self.root_buddy.get_subscriptions().mouse_click {
            let point = event.get_point();
            let mut fire = false;
            let maybe_render_result = self.root_buddy.get_last_render_result();

            // Don't pass on any click events until the component has been
            // rendered for the first time.
            if let Some(render_result) = maybe_render_result {
                // If we should filter mouse actions, we need to do an additional check
                if render_result.filter_mouse_actions {
                    fire = render_result
                        .drawn_region
                        .is_inside(point.get_x(), point.get_y());
                } else {
                    fire = true;
                }
            }

            if fire {
                self.root_component
                    .on_mouse_click(event, &mut self.root_buddy);
                self.work_after_events();
            }
        }
        // TODO Handle mouse click out
    }
}

impl Drop for Application {
    fn drop(&mut self) {
        self.root_component.on_detach();
    }
}
