use crate::*;

pub struct SimpleFlatColorComponent {
    color: Color,
}

impl SimpleFlatColorComponent {
    pub fn new(color: Color) -> Self {
        Self { color }
    }
}

impl Component for SimpleFlatColorComponent {
    fn on_attach(&mut self, _buddy: &mut dyn ComponentBuddy) {}

    fn render(
        &mut self,
        renderer: &Renderer,
        _buddy: &mut dyn ComponentBuddy,
        _force: bool,
    ) -> RenderResult {
        renderer.clear(self.color);
        entire_render_result()
    }
}
