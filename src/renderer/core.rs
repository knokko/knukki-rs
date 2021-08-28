use crate::*;

impl Renderer {
    /// Starts this `Renderer`. The `Application` is supposed to call this method each time before
    /// it starts rendering its components.
    ///
    /// Currently, this method will only ensure that the viewport and scissor are up-to-date.
    pub fn start(&self) {
        self.apply_viewport_and_scissor();
    }

    /// Sets the viewport and scissor of the rendering context (probably OpenGL) to the current
    /// value of `self.get_viewport()` and `self.get_scissor()` respectively.
    #[cfg(not(feature = "golem_rendering"))]
    pub fn apply_viewport_and_scissor(&self) {
        // There is nothing to be done without a Golem context
    }

    /// Sets the color of all pixels within the current viewport and scissor to the given `Color`.
    #[allow(unused_variables)]
    #[cfg(not(feature = "golem_rendering"))]
    pub fn clear(&self, color: Color) {
        // There is nothing to be done without a Golem context
    }

    /// Uses the given *FragmentOnlyShader* to fill the rectangular region defined by *min_x*,
    /// *min_y*, *max_x*, and *max_y* (each of them should be between 0.0 and 1.0) using the given
    /// *parameters* (typically uniform variables). If you don't want to draw on the entire
    /// rectangular region, you can let the fragment shader *discard* those pixels.
    #[allow(unused_variables)]
    #[cfg(not(feature = "golem_rendering"))]
    pub fn apply_fragment_shader(
        &self, min_x: f32, min_y: f32, max_x: f32, max_y: f32,
        shader: &FragmentOnlyShader, parameters: FragmentOnlyDrawParameters
    ) {
        // There is nothing to be done without a Golem context
    }

    /// Gets the current viewport region of this `Renderer`. The drawing operations of components
    /// will be scaled and translated to fit inside this region.
    pub fn get_viewport(&self) -> RenderRegion {
        let viewport_stack = self.viewport_stack.borrow();
        *viewport_stack
            .last()
            .expect("Viewport stack is never empty")
    }

    /// Gets the current scissor region of this `Renderer`. Components won't be able to draw
    /// anything outside this region.
    ///
    /// When components draw something that is partially outside this region, the pixels outside
    /// this region simply won't be affected, but the pixels inside this region will change
    /// normally.
    pub fn get_scissor(&self) -> RenderRegion {
        let scissor_stack = self.scissor_stack.borrow();
        *scissor_stack.last().expect("Scissor stack is never empty")
    }

    /// Gets the `TextRenderer` of this `Renderer`. As the name suggests, the text renderer can be
    /// used to draw text.
    pub fn get_text_renderer(&self) -> &TextRenderer {
        &self.text_renderer
    }

    /// Shrinks the viewport (and scissor) by the given amounts, calls the `render_function`, and
    /// thereafter restores the viewport and scissor.
    ///
    /// ## Edge case
    /// But, if the shrunk viewport or scissor would have a width or height of 0, the
    /// `render_function` will **not** be called, and this method will return `None`.
    ///
    /// ## Motivation
    /// The motivation behind this function is to help menu components: they typically want to
    /// render components inside an area that is smaller than the entire viewport. So, they call
    /// this method, use the `(min_x, min_y, max_x, max_y)` parameters to define the region in
    /// which the component is allowed to render, and call the `render` method of the component in
    /// the `render_function`.
    ///
    /// ## Details
    /// The `new_viewport` will be equal to `old_viewport.child_region(min_x, min_y, max_x, max_y)`
    /// and the `new_scissor` will be equal to `old_scissor.intersection(new_viewport)`.
    pub fn push_viewport<R>(
        &self,
        min_x: f32,
        min_y: f32,
        max_x: f32,
        max_y: f32,
        render_function: impl FnOnce() -> R,
    ) -> Option<R> {
        let parent_viewport = self.get_viewport();
        let maybe_child_viewport = parent_viewport.child_region(min_x, min_y, max_x, max_y);

        if let Some(child_viewport) = maybe_child_viewport {
            let parent_scissor = self.get_scissor();
            let maybe_child_scissor = parent_scissor.intersection(child_viewport);

            // Don't bother calling the render function if there would be an empty scissor
            if let Some(child_scissor) = maybe_child_scissor {
                // Push the viewport
                let mut viewport_stack = self.viewport_stack.borrow_mut();
                viewport_stack.push(child_viewport);
                drop(viewport_stack);

                // Push the scissor
                let mut scissor_stack = self.scissor_stack.borrow_mut();
                scissor_stack.push(child_scissor);
                drop(scissor_stack);

                // Make sure the viewport and scissor are actually used
                self.apply_viewport_and_scissor();

                // Call the render function
                let result = render_function();

                // Pop the viewport and scissor
                let mut viewport_stack = self.viewport_stack.borrow_mut();
                viewport_stack.pop();
                drop(viewport_stack);
                let mut scissor_stack = self.scissor_stack.borrow_mut();
                scissor_stack.pop();
                drop(scissor_stack);

                self.apply_viewport_and_scissor();

                // Return the result
                Some(result)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Calls the `render_function`, but ensures that the region `(min_x, min_y, max_x, max_y)`
    /// will **not** be affected by the render function.
    ///
    /// ## Effects on drawing
    /// If the render function renders something entirely inside the region, it will be drawn
    /// normally. If the render function renders something entirely outside the region, it will
    /// have no effect (but might waste some graphics card time). If it renders something that is
    /// partially inside the region, only the part inside the region will be committed.
    ///
    /// ## Optional behavior
    /// If there would be no drawing region left (for instance when nesting 2 calls with given
    /// regions that have no overlap at all), the `render_function` will **not** be called, and this
    /// method will return `None`.
    ///
    /// ## Motivation
    /// The motivation behind this function is to help menu components and the *wrapper* with doing
    /// partial redraws. For instance, the *wrapper* could use a scissor when only a small part of
    /// the window needs to be redrawn. (In particular, I am considering adding custom mouses.
    /// Using this scissor system, the *wrapper* could instruct the application to only
    /// redraw the area that used to be behind the custom mouse). It could also be helpful to
    /// layered menu components: when a small component is (re)moved on the front layer, only the
    /// part on the background layer behind that component needs to be redrawn.
    ///
    /// ## Details
    /// The `new_scissor` will be equal to
    /// `old_scissor.intersection(viewport.child_region(min_x, min_y, max_x, max_y))`.
    /// The viewport will **not** be affected by this method.
    ///
    /// ## Result
    /// If the `render_function` is called, its result will be returned (inside a `Some`). If not,
    /// this method will return `None`.
    pub fn push_scissor<R>(
        &self,
        min_x: f32,
        min_y: f32,
        max_x: f32,
        max_y: f32,
        render_function: impl FnOnce() -> R,
    ) -> Option<R> {
        let old_scissor = self.get_scissor();
        let viewport = self.get_viewport();
        let maybe_new_scissor = viewport.child_region(min_x, min_y, max_x, max_y);
        if let Some(new_scissor) = maybe_new_scissor {
            if let Some(combined_scissor) = old_scissor.intersection(new_scissor) {
                let mut scissor_stack = self.scissor_stack.borrow_mut();
                scissor_stack.push(combined_scissor);
                drop(scissor_stack);

                self.apply_viewport_and_scissor();

                let result = render_function();

                let mut scissor_stack = self.scissor_stack.borrow_mut();
                scissor_stack.pop();
                drop(scissor_stack);

                self.apply_viewport_and_scissor();

                return Some(result);
            }
        }

        None
    }

    /// (Re-)sets the viewport and scissor of this `Renderer` to `new_viewport`. This will clear
    /// the entire viewport stack and scissor stack.
    ///
    /// This method requires a mutable reference to `self` because it is intended to be used only
    /// by the *wrapper*, which should call this before the `render` method of the `Application`,
    /// to specify where the `Application` should be rendered.
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

    #[test]
    fn test_push_viewport() {
        let outer_region = RenderRegion::between(50, 50, 250, 250);
        let middle_region = RenderRegion::between(100, 50, 200, 250);
        let inner_region = RenderRegion::between(125, 75, 175, 225);

        let renderer = test_renderer(outer_region);
        assert_eq!(outer_region, renderer.get_viewport());
        assert_eq!(outer_region, renderer.get_scissor());

        let mut counter = 0;
        renderer
            .push_viewport(0.25, 0.0, 0.75, 1.0, || {
                counter += 1;
                assert_eq!(middle_region, renderer.get_viewport());
                assert_eq!(middle_region, renderer.get_scissor());

                renderer
                    .push_viewport(0.25, 0.125, 0.75, 0.875, || {
                        assert_eq!(1, counter);
                        counter += 1;
                        assert_eq!(inner_region, renderer.get_viewport());
                        assert_eq!(inner_region, renderer.get_scissor());

                        // And push onto an empty viewport
                        assert!(renderer
                            .push_viewport(0.001, 0.001, 0.002, 0.002, || {
                                unreachable!();
                            })
                            .is_none());
                    })
                    .unwrap();

                assert_eq!(2, counter);
                assert_eq!(middle_region, renderer.get_viewport());
                assert_eq!(middle_region, renderer.get_scissor());
            })
            .unwrap();
        assert_eq!(2, counter);

        assert_eq!(outer_region, renderer.get_viewport());
        assert_eq!(outer_region, renderer.get_scissor());
    }

    #[test]
    fn test_push_scissor() {
        let viewport = RenderRegion::with_size(50, 100, 400, 300);
        let bottom_left = RenderRegion::with_size(50, 100, 200, 150);
        let partial_middle = RenderRegion::with_size(150, 175, 100, 75);

        let renderer = test_renderer(viewport);

        assert!(renderer
            .push_scissor(0.0, 0.0, 0.5, 0.5, || {
                assert_eq!(viewport, renderer.get_viewport());
                assert_eq!(bottom_left, renderer.get_scissor());

                // Should return None when the scissors have an empty intersection
                assert!(renderer
                    .push_scissor(0.5, 0.5, 1.0, 1.0, || {
                        unreachable!();
                    })
                    .is_none());
                assert!(renderer
                    .push_scissor(0.6, 0.6, 0.8, 0.7, || {
                        unreachable!();
                    })
                    .is_none());

                // Pushing the same scissor twice has no extra effect
                assert!(renderer
                    .push_scissor(0.0, 0.0, 0.5, 0.5, || {
                        assert_eq!(viewport, renderer.get_viewport());
                        assert_eq!(bottom_left, renderer.get_scissor());
                    })
                    .is_some());

                assert!(renderer
                    .push_scissor(0.25, 0.25, 0.75, 0.75, || {
                        assert_eq!(viewport, renderer.get_viewport());
                        assert_eq!(partial_middle, renderer.get_scissor());
                    })
                    .is_some());

                assert_eq!(bottom_left, renderer.get_scissor());
            })
            .is_some());

        assert_eq!(viewport, renderer.get_viewport());
        assert_eq!(viewport, renderer.get_scissor());
    }

    #[test]
    fn test_combine_viewport_and_scissor() {
        let outer_viewport = RenderRegion::with_size(0, 0, 1000, 1000);
        let inner_viewport = RenderRegion::with_size(250, 250, 500, 500);
        let left_inner_viewport = RenderRegion::with_size(250, 250, 350, 500);
        let inner_left_viewport = RenderRegion::with_size(250, 250, 300, 500);

        let renderer = test_renderer(outer_viewport);

        renderer
            .push_scissor(0.0, 0.0, 0.6, 1.0, || {
                renderer
                    .push_viewport(0.25, 0.25, 0.75, 0.75, || {
                        assert_eq!(inner_viewport, renderer.get_viewport());
                        assert_eq!(left_inner_viewport, renderer.get_scissor());
                    })
                    .unwrap();
            })
            .unwrap();

        renderer
            .push_viewport(0.25, 0.25, 0.75, 0.75, || {
                renderer
                    .push_scissor(0.0, 0.0, 0.6, 1.0, || {
                        assert_eq!(inner_viewport, renderer.get_viewport());
                        assert_eq!(inner_left_viewport, renderer.get_scissor());
                    })
                    .unwrap();
            })
            .unwrap();

        renderer
            .push_scissor(0.0, 0.0, 0.6, 1.0, || {
                assert!(renderer
                    .push_viewport(0.8, 0.8, 0.9, 0.9, || {
                        unreachable!();
                    })
                    .is_none());
            })
            .unwrap();
    }

    // TODO Write an example that uses the push_scissor method. Note: probably requires WrapperComponent first
}
