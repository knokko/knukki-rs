use crate::*;

use golem::Context;

pub struct FlatColorComponent {
    color: Color
}

impl FlatColorComponent {
    pub fn new(color: Color) -> Self {
        Self { color }
    }
}

impl Component for FlatColorComponent {
    fn on_attach(&mut self, _buddy: &mut dyn ComponentBuddy) {}

    fn render(&mut self, golem: &Context, _region: RenderRegion, _buddy: &mut dyn ComponentBuddy) -> RenderResult {
        golem.set_clear_color(
            self.color.get_red_float(), 
            self.color.get_green_float(), 
            self.color.get_blue_float(), 
            self.color.get_alpha_float()
        );
        golem.clear();
        RenderResult::entire()
    }
}