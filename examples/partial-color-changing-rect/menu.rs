use knukki::*;

pub const EXAMPLE_NAME: &'static str = "partial-color-changing-rect";

pub fn create_app() -> Application {
    let component = ColorChangingRectComponent {
        red: 200,
        green: 100,
        blue: 0,
    };

    Application::new(Box::new(component))
}

struct ColorChangingRectComponent {
    red: u8,
    green: u8,
    blue: u8,
}

impl Component for ColorChangingRectComponent {
    fn on_attach(&mut self, buddy: &mut dyn ComponentBuddy) {
        buddy.subscribe_mouse_move();
        buddy.subscribe_mouse_enter();
        buddy.subscribe_mouse_leave();
    }

    fn render(
        &mut self,
        renderer: &Renderer,
        _buddy: &mut dyn ComponentBuddy,
        _force: bool,
    ) -> RenderResult {
        renderer.push_viewport(0.2, 0.2, 0.8, 0.8, || {
            renderer.clear(Color::rgb(self.red, self.green, self.blue));
        });

        Ok(RenderResultStruct {
            filter_mouse_actions: true,
            drawn_region: Box::new(RectangularDrawnRegion::new(0.2, 0.2, 0.8, 0.8)),
        })
    }

    fn on_mouse_move(&mut self, event: MouseMoveEvent, buddy: &mut dyn ComponentBuddy) {
        let dx = event.get_delta_x();
        if dx > 0.0 {
            self.red = self.red.wrapping_add((dx * 100.0) as u8);
        } else {
            self.green = self.green.wrapping_add((dx * -80.0) as u8);
        }
        let dy = event.get_delta_y();
        if dy > 0.0 {
            self.blue = self.blue.wrapping_add((dy * 100.0) as u8);
        } else {
            let delta = (dy * -40.0) as u8;
            self.red = self.red.wrapping_add(delta);
            self.green = self.green.wrapping_add(delta);
            self.blue = self.blue.wrapping_add(delta);
        }
        buddy.request_render();
    }

    fn on_mouse_enter(&mut self, _event: MouseEnterEvent, buddy: &mut dyn ComponentBuddy) {
        self.red = 200;
        self.green = 200;
        self.blue = 200;
        buddy.request_render();
    }

    fn on_mouse_leave(&mut self, _event: MouseLeaveEvent, buddy: &mut dyn ComponentBuddy) {
        self.red = 50;
        self.green = 50;
        self.blue = 50;
        buddy.request_render();
    }
}
