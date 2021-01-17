use crate::*;
use golem::*;

impl Renderer {
    pub fn new(context: Context, initial_viewport: RenderRegion) -> Self {
        Self {
            context,
            viewport_stack: vec![initial_viewport],
            scissor_stack: vec![initial_viewport],
        }
    }

    pub fn get_context(&self) -> &Context {
        &self.context
    }

    pub fn apply_viewport_and_scissor(&self) {
        let viewport = self.viewport_stack.last().expect("viewport stack is never empty");
        viewport.set_viewport(&self.context);
        let scissor = self.scissor_stack.last().expect("scissor stack is never empty");
        scissor.set_scissor(&self.context);
    }
}
