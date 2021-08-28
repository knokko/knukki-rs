use crate::*;

/// A component that will draw a simple circle at its position. It has a `base_color` and a
/// `hover_color`. It will fill the circle with the `hover_color` while a `Mouse` is hovering over
/// it. If not, it will fill the circle with the `base_color`.
///
/// This is clearly not a useful component in a real application, but it is a nice example because
/// it demonstrates how to avoid distortion and how to use hover mechanics correctly.
#[allow(dead_code)] // The fields are only used when golem rendering is enabled
pub struct HoverColorCircleComponent {
    base_color: Color,
    hover_color: Color,
    shader: FragmentOnlyShader
}

impl HoverColorCircleComponent {
    pub fn new(base_color: Color, hover_color: Color) -> Self {
        Self {
            base_color,
            hover_color,
            shader: create_fragment_only_shader()
        }
    }
}

fn create_fragment_only_shader() -> FragmentOnlyShader {
    FragmentOnlyShader::new(FragmentOnlyShaderDescription {
        source_code: "
            void main() {
                vec2 radius = floatVector1.xy;
                float dx = (innerPosition.x - 0.5) / radius.x;
                float dy = (innerPosition.y - 0.5) / radius.y;
                if (dx * dx + dy * dy <= 1.0) {
                    gl_FragColor = color1;
                } else {
                    discard;
                }
            }
        ".to_string(),
        num_float_matrices: 0,
        num_colors: 1,
        num_float_vectors: 1,
        num_int_vectors: 0,
        num_floats: 0,
        num_ints: 0
    })
}

impl Component for HoverColorCircleComponent {
    fn on_attach(&mut self, buddy: &mut dyn ComponentBuddy) {
        buddy.subscribe_mouse_enter();
        buddy.subscribe_mouse_leave();
    }

    fn render(
        &mut self,
        renderer: &Renderer,
        #[allow(unused_variables)] // The buddy parameter is only used when golem_rendering is enabled
        buddy: &mut dyn ComponentBuddy,
        _force: bool,
    ) -> RenderResult {
        // The first challenge is to avoid distortion: if the *region* is rectangular rather than
        // square, we will ignore a part of it such that a square part remains, and use that.
        let ar = renderer.get_viewport().get_aspect_ratio();
        let used_width = 0.5 / ar.max(1.0);
        let used_height = 0.5 / (1.0 / ar).max(1.0);

        let drawn_region =
            OvalDrawnRegion::new(Point::new(0.5, 0.5), used_width, used_height);

        // Now that we know the exact region in which we render, we can determine whether any mouse
        // is hovering over that region
        let is_hovering = buddy.get_local_mouses().iter().any(|mouse| {
            match buddy.get_mouse_position(*mouse) {
                Some(position) => drawn_region.is_inside(position),
                None => {
                    // Weird and shouldn't happen, but not a critical problem
                    debug_assert!(false);
                    false
                }
            }
        });

        let color = match is_hovering {
            true => self.hover_color,
            false => self.base_color,
        };

        renderer.apply_fragment_shader(
            0.0, 0.0, 1.0, 1.0, &self.shader, FragmentOnlyDrawParameters {
                colors: &[color],
                float_vectors: &[[used_width, used_height, 0.0, 0.0]],
                ..FragmentOnlyDrawParameters::default()
            }
        );

        Ok(RenderResultStruct {
            drawn_region: Box::new(drawn_region),
            filter_mouse_actions: true,
        })
    }

    fn on_mouse_enter(&mut self, _event: MouseEnterEvent, buddy: &mut dyn ComponentBuddy) {
        buddy.request_render();
    }

    fn on_mouse_leave(&mut self, _event: MouseLeaveEvent, buddy: &mut dyn ComponentBuddy) {
        buddy.request_render();
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn test_render_returned_region() {
        let mut component =
            HoverColorCircleComponent::new(Color::rgb(10, 20, 30), Color::rgb(100, 110, 120));
        let mut buddy = RootComponentBuddy::new();
        buddy.set_mouse_store(Rc::new(RefCell::new(MouseStore::new())));

        let square_region = RenderRegion::with_size(10, 20, 50, 50);
        let square_result = component
            .render(&test_renderer(square_region), &mut buddy, true)
            .unwrap()
            .drawn_region;
        assert!(Point::new(0.0, 0.0).nearly_equal(Point::new(
            square_result.get_left(),
            square_result.get_bottom()
        )));
        assert!(Point::new(1.0, 1.0).nearly_equal(Point::new(
            square_result.get_right(),
            square_result.get_top()
        )));

        let wide_region = RenderRegion::with_size(10, 20, 100, 50);
        let wide_result = component
            .render(&test_renderer(wide_region), &mut buddy, true)
            .unwrap()
            .drawn_region;
        assert!(Point::new(0.25, 0.0)
            .nearly_equal(Point::new(wide_result.get_left(), wide_result.get_bottom())));
        assert!(Point::new(0.75, 1.0)
            .nearly_equal(Point::new(wide_result.get_right(), wide_result.get_top())));

        let high_region = RenderRegion::with_size(10, 20, 50, 100);
        let high_result = component
            .render(&test_renderer(high_region), &mut buddy, true)
            .unwrap()
            .drawn_region;
        assert!(Point::new(0.0, 0.25)
            .nearly_equal(Point::new(high_result.get_left(), high_result.get_bottom())));
        assert!(Point::new(1.0, 0.75)
            .nearly_equal(Point::new(high_result.get_right(), high_result.get_top())));
    }
}
