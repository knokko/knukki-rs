use knukki::*;

fn main() {
    let component = TestComponent {
        red: 100,
        green: 200,
    };
    let app = Application::new(Box::new(component));

    start(app, "Hello knukki");
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
        _force: bool
    ) -> RenderResult {
        renderer.get_context().set_clear_color(self.red as f32 / 255.0, self.green as f32 / 255.0, 0.4, 1.0);
        renderer.get_context().clear();
        entire_render_result()
    }

    fn on_mouse_click(&mut self, _event: MouseClickEvent, buddy: &mut dyn ComponentBuddy) {
        self.red = self.red.wrapping_add(100);
        self.green = self.green.wrapping_add(17);
        buddy.request_render();
    }
}
