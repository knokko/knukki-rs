use crate::*;
use golem::*;

impl RendererStruct {
    pub fn new(context: Context) -> Self {
        Self { context }
    }

    pub fn get_context(&self) -> &Context {
        &self.context
    }

    // TODO Create some kind of stack for the viewport and scissor
    // I guess some kind of closure system would be best: push; run closure; pop;
    // Pushing a scissor should be affected by the last viewport
}

//pub type Renderer<'a> = &'a GolemRenderer;