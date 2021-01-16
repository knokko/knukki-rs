use golem::*;

pub struct GolemRenderer {
    context: Context
}

impl GolemRenderer {
    pub fn new(context: Context) -> Self {
        Self { context }
    }

    pub fn get_context(&self) -> &Context {
        &self.context
    }
}

pub type Renderer<'a> = &'a GolemRenderer;