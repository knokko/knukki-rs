use knukki::*;

pub const EXAMPLE_NAME: &'static str = "color-changing-rect-menu";

pub fn create_app() -> Application {
    let mut menu = SimpleFlatMenu::new(Some(Color::rgb(50, 150, 100)));

    menu.add_component(
        Box::new(ColorChangingRectComponent {
            red: 200,
            green: 100,
            blue: 0,
        }),
        ComponentDomain::between(0.1, 0.1, 0.8, 0.4),
    );
    menu.add_component(
        Box::new(ColorChangingRectComponent {
            red: 50,
            green: 200,
            blue: 150,
        }),
        ComponentDomain::between(0.5, 0.5, 0.9, 0.9),
    );
    menu.add_component(
        Box::new(ColorChangingRectComponent {
            red: 200,
            green: 0,
            blue: 150,
        }),
        ComponentDomain::between(0.05, 0.7, 0.4, 0.95),
    );

    Application::new(Box::new(menu))
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
        renderer.clear(Color::rgb(self.red, self.green, self.blue));
        entire_render_result()
    }

    fn on_mouse_move(&mut self, event: MouseMoveEvent, buddy: &mut dyn ComponentBuddy) {
        let dx = event.get_delta_x();
        if dx > 0.0 {
            self.red = self.red.wrapping_add((dx * 50.0) as u8);
        } else {
            self.green = self.green.wrapping_add((dx * -40.0) as u8);
        }
        let dy = event.get_delta_y();
        if dy > 0.0 {
            self.blue = self.blue.wrapping_add((dy * 30.0) as u8);
        } else {
            let delta = (dy * -30.0) as u8;
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
