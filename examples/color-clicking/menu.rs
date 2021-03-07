use knukki::*;

pub const EXAMPLE_NAME: &'static str = "color-clicking";

pub fn create_app() -> Application {
    let component = TestComponent {
        red: 100,
        green: 200,
    };

    Application::new(Box::new(component))
}

struct TestComponent {
    red: u8,
    green: u8,
}

impl Component for TestComponent {
    fn on_attach(&mut self, buddy: &mut dyn ComponentBuddy) {
        buddy.subscribe_mouse_click();
    }

    fn render(
        &mut self,
        renderer: &Renderer,
        _buddy: &mut dyn ComponentBuddy,
        _force: bool,
    ) -> RenderResult {
        renderer.clear(Color::rgb(self.red, self.green, 100));
        entire_render_result()
    }

    fn on_mouse_click(&mut self, _event: MouseClickEvent, buddy: &mut dyn ComponentBuddy) {
        self.red = self.red.wrapping_add(100);
        self.green = self.green.wrapping_add(17);
        buddy.request_render();
    }
}
