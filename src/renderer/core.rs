use crate::*;

impl Renderer {
    pub fn start(&self) {
        self.apply_viewport_and_scissor();
    }

    #[cfg(not(feature = "golem_rendering"))]
    pub fn apply_viewport_and_scissor(&self) {
        // There is nothing to be done without a Golem context
    }

    // TODO Clear color method

    pub fn get_viewport(&self) -> RenderRegion {
        *self.viewport_stack.last().expect("Viewport stack is never empty")
    }

    pub fn get_scissor(&self) -> RenderRegion {
        *self.scissor_stack.last().expect("Scissor stack is never empty")
    }

    /// Note: to ensure glClear is consistent, this will also push a scissor
    pub fn push_viewport<R>(&self, min_x: f32, min_y: f32, max_x: f32, max_y: f32, use_function: impl FnOnce() -> R) -> R{
        let parent_viewport = self.get_viewport();
        let child_viewport = parent_viewport.child_region(
            min_x, min_y, max_x, max_y
        );
        // TODO Finish the implementation
        use_function()
    }

    pub fn reset_viewport(&mut self, new_viewport: RenderRegion) {
        self.viewport_stack.clear();
        self.scissor_stack.clear();
        self.viewport_stack.push(new_viewport);
        self.scissor_stack.push(new_viewport);
        self.apply_viewport_and_scissor();
    }

    // TODO Also test reset_viewport

    // TODO Write a test for push_viewport

    // TODO Add a push_scissor method, and write a test for it
}