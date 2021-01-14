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
        #[cfg(feature = "golem_rendering")] golem: &golem::Context,
        _region: RenderRegion,
        _buddy: &mut dyn ComponentBuddy,
        _force: bool,
    ) -> RenderResult {
        #[cfg(feature = "golem_rendering")]
        {
            golem.set_clear_color(
                self.color.get_red_float(),
                self.color.get_green_float(),
                self.color.get_blue_float(),
                self.color.get_alpha_float(),
            );
            golem.clear();
        }
        entire_render_result()
    }
}
