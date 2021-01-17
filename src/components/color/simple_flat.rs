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
        #[cfg(feature = "golem_rendering")]
        {
            renderer.get_context().set_clear_color(
                self.color.get_red_float(),
                self.color.get_green_float(),
                self.color.get_blue_float(),
                self.color.get_alpha_float(),
            );
            renderer.get_context().clear();
        }
        entire_render_result()
    }
}
