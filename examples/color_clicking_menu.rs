use golem::Context;
use knukki::*;

fn main() {
    let mut menu = SimpleFlatMenu::new();
    menu.add_component(
        Box::new(TestComponent { red: 100, green: 0 }),
        ComponentDomain::between(0.1, 0.1, 0.7, 0.3)
    );
    menu.add_component(
        Box::new(TestComponent { red: 100, green: 200 }),
        ComponentDomain::between(0.3, 0.5, 0.6, 0.9)
    );

    let app = Application::new(Box::new(menu));
    start(app, "Click the colors");
}

struct TestComponent {
    red: u8,
    green: u8
}

impl Component for TestComponent {
    fn on_attach(&mut self, buddy: &mut dyn ComponentBuddy) {
        buddy.subscribe_mouse_click();
    }

    fn on_mouse_click(&mut self, _event: MouseClickEvent, buddy: &mut dyn ComponentBuddy) {
        self.red = self.red.wrapping_add(100);
        self.green = self.green.wrapping_add(17);
        buddy.request_render();
    }

    fn render(&mut self, golem: &Context, _region: RenderRegion, _buddy: &mut dyn ComponentBuddy) -> RenderResult {
        golem.set_clear_color(self.red as f32 / 255.0, self.green as f32 / 255.0, 0.4, 1.0);
        golem.clear();
        RenderResult::entire()
    }
}