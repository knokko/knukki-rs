use crate::*;
use golem::*;
use std::cell::RefCell;

impl Renderer {
    pub fn new(context: Context, initial_viewport: RenderRegion) -> Self {
        Self {
            context,
            viewport_stack: RefCell::new(vec![initial_viewport]),
            scissor_stack: RefCell::new(vec![initial_viewport]),
        }
    }

    pub fn clear(&self, color: Color) {
        self.context.set_clear_color(
            color.get_red_float(), color.get_green_float(),
            color.get_blue_float(), color.get_alpha_float()
        );
        self.context.clear();
    }

    pub fn get_context(&self) -> &Context {
        &self.context
    }

    pub fn apply_viewport_and_scissor(&self) {
        self.get_viewport().set_viewport(&self.context);
        self.get_scissor().set_scissor(&self.context);
    }
}
