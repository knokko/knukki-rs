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

    pub fn get_context(&self) -> &Context {
        &self.context
    }

    pub fn apply_viewport_and_scissor(&self) {
        self.get_viewport().set_viewport(&self.context);
        self.get_scissor().set_scissor(&self.context);
    }
}
