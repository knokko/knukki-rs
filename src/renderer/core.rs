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
        let viewport_stack = self.viewport_stack.borrow();
        *viewport_stack.last().expect("Viewport stack is never empty")
    }

    pub fn get_scissor(&self) -> RenderRegion {
        let scissor_stack = self.scissor_stack.borrow();
        *scissor_stack.last().expect("Scissor stack is never empty")
    }

    /// Note: to ensure glClear is consistent, this will also push a scissor
    pub fn push_viewport<R>(&self, min_x: f32, min_y: f32, max_x: f32, max_y: f32, use_function: impl FnOnce() -> R) -> R{
        let parent_viewport = self.get_viewport();
        let child_viewport = parent_viewport.child_region(
            min_x, min_y, max_x, max_y
        );

        // Push the viewport
        let mut viewport_stack = self.viewport_stack.borrow_mut();
        viewport_stack.push(child_viewport);
        drop(viewport_stack);

        // TODO Push the scissor

        let result = use_function();

        // Pop the viewport and scissor
        let mut viewport_stack = self.viewport_stack.borrow_mut();
        viewport_stack.pop();
        let mut scissor_stack = self.scissor_stack.borrow_mut();
        scissor_stack.pop();

        // Return the result
        result
    }

    pub fn reset_viewport(&mut self, new_viewport: RenderRegion) {
        let mut viewport_stack = self.viewport_stack.borrow_mut();
        let mut scissor_stack = self.scissor_stack.borrow_mut();

        viewport_stack.clear();
        scissor_stack.clear();
        viewport_stack.push(new_viewport);
        scissor_stack.push(new_viewport);

        drop(viewport_stack);
        drop(scissor_stack);

        self.apply_viewport_and_scissor();
    }

    // TODO Add a push_scissor method, and write a test for it
}

#[cfg(test)]
mod tests {

    use crate::*;

    #[test]
    fn test_reset_viewport() {
        let region1 = RenderRegion::with_size(1, 2, 3, 4);
        let region2 = RenderRegion::with_size(5, 6, 7, 8);
        let region3 = RenderRegion::with_size(9, 10, 11, 12);

        let mut renderer = test_renderer(region1);
        assert_eq!(region1, renderer.get_viewport());
        assert_eq!(region1, renderer.get_scissor());

        renderer.reset_viewport(region2);
        assert_eq!(region2, renderer.get_viewport());
        assert_eq!(region2, renderer.get_scissor());

        renderer.push_viewport(0.1, 0.1, 0.8, 0.7, || {});
        renderer.reset_viewport(region3);
        assert_eq!(region3, renderer.get_viewport());
        assert_eq!(region3, renderer.get_scissor());
    }

    // TODO Also test pushing until an empty viewport

    #[test]
    fn test_push_viewport() {
        let outer_region = RenderRegion::between(50, 50, 250, 250);
        let middle_region = RenderRegion::between(100, 50, 200, 250);
        let inner_region = RenderRegion::between(125, 75, 175, 225);

        let renderer = test_renderer(outer_region);
        assert_eq!(outer_region, renderer.get_viewport());
        assert_eq!(outer_region, renderer.get_scissor());

        let mut counter = 0;
        renderer.push_viewport(0.25, 0.0, 0.75, 1.0, || {
            counter += 1;
            assert_eq!(middle_region, renderer.get_viewport());
            assert_eq!(middle_region, renderer.get_scissor());

            renderer.push_viewport(0.25, 0.125, 0.75, 0.875, || {
                assert_eq!(1, counter);
                counter += 1;
                assert_eq!(inner_region, renderer.get_viewport());
                assert_eq!(inner_region, renderer.get_scissor());
            });

            assert_eq!(2, counter);
            assert_eq!(middle_region, renderer.get_viewport());
            assert_eq!(middle_region, renderer.get_scissor());
        });
        assert_eq!(2, counter);

        assert_eq!(middle_region, renderer.get_viewport());
        assert_eq!(middle_region, renderer.get_scissor());
    }
}