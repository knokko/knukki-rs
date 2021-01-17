use knukki::*;

fn main() {
    let mut menu = SimpleFlatMenu::new(Some(Color::rgb(0, 0, 0)));
    menu.add_component(
        Box::new(ColorChangingRectComponent { red: 200, green: 100, blue: 0, id: 1 }),
        ComponentDomain::between(0.1, 0.1, 0.8, 0.4)
    );
    menu.add_component(
        Box::new(ColorChangingRectComponent { red: 50, green: 200, blue: 150, id: 2 }),
        ComponentDomain::between(0.5, 0.5, 0.9, 0.9)
    );
    menu.add_component(
        Box::new(ColorChangingRectComponent { red: 200, green: 0, blue: 150, id: 3 }),
        ComponentDomain::between(0.05, 0.7, 0.4, 0.95)
    );
    let application = Application::new(Box::new(menu));
    start(application, "Color changing rect menu");
}

struct ColorChangingRectComponent {

    red: u8,
    green: u8,
    blue: u8,

    id: u8
}

impl Component for ColorChangingRectComponent {
    fn on_attach(&mut self, buddy: &mut dyn ComponentBuddy) {
        buddy.subscribe_mouse_move();
        buddy.subscribe_mouse_enter();
        buddy.subscribe_mouse_leave();
    }

    fn render(
        &mut self,
        renderer: Renderer,
        _region: RenderRegion,
        _buddy: &mut dyn ComponentBuddy,
        _force: bool
    ) -> RenderResult {
        renderer.get_context().set_clear_color(
            self.red as f32 / 255.0,
            self.green as f32 / 255.0,
            self.blue as f32 / 255.0,
            1.0
        );
        renderer.get_context().clear();

        Ok(RenderResultStruct {
            filter_mouse_actions: false,
            drawn_region: Box::new(RectangularDrawnRegion::new(0.2, 0.2, 0.8, 0.8))
        })
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