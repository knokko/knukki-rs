use crate::*;

/// A component that will draw a simple circle at its position. It has a `base_color` and a
/// `hover_color`. It will fill the circle with the `hover_color` while a `Mouse` is hovering over
/// it. If not, it will fill the circle with the `base_color`.
///
/// This is clearly not a useful component in a real application, but it is a nice example because
/// it demonstrates how to avoid distortion and how to use hover mechanics correctly.
pub struct HoverColorCircleComponent {
    base_color: Color,
    hover_color: Color,
}

impl HoverColorCircleComponent {
    pub fn new(base_color: Color, hover_color: Color) -> Self {
        Self { base_color, hover_color }
    }
}

impl Component for HoverColorCircleComponent {
    fn on_attach(&mut self, buddy: &mut dyn ComponentBuddy) {
        buddy.subscribe_mouse_enter();
        buddy.subscribe_mouse_leave();
    }

    fn render(
        &mut self,
        #[cfg(feature = "golem_rendering")]
        golem: &golem::Context,
        region: RenderRegion,
        buddy: &mut dyn ComponentBuddy,
        _force: bool
    ) -> RenderResult {

        // The first challenge is to avoid distortion: if the *region* is rectangular rather than
        // square, we will ignore a part of it such that a square part remains, and use that.
        let ar = region.get_aspect_ratio();
        let used_width = 1.0 / ar.max(1.0);
        let used_height = 1.0 / (1.0 / ar).max(1.0);

        let drawn_region = OvalDrawnRegion::new(
            Point::new(0.5, 0.5), used_width * 0.5, used_height * 0.5
        );

        // Now that we know the exact region in which we render, we can determine whether any mouse
        // is hovering over that region
        let is_hovering = buddy.get_local_mouses().iter().any(|mouse| {
           match buddy.get_mouse_position(*mouse) {
               Some(position) => {
                   drawn_region.is_inside(position)
               }, None => {
                   // Weird and shouldn't happen, but not a critical problem
                   debug_assert!(false);
                   false
               }
           }
        });

        // If the golem rendering feature is enabled, we should also draw the circle
        #[cfg(feature = "golem_rendering")]
        {
            use golem::*;
            // TODO Optimize this by storing the model and shaders rather than recreating them
            // all the time

            // I will use the fragment shader to ensure the quad looks like a circle
            #[rustfmt::skip]
                let quad_vertices = [
                -1.0, -1.0,    1.0, -1.0,    1.0, 1.0,    -1.0, 1.0,
            ];
            #[rustfmt::skip]
            let quad_indices = [
                0, 1, 2, 2, 3, 0
            ];

            #[rustfmt::skip]
            let shader_description = ShaderDescription {
                vertex_input: &[
                    Attribute::new("position", AttributeType::Vector(Dimension::D2))
                ],
                fragment_input: &[
                    Attribute::new("passPosition", AttributeType::Vector(Dimension::D2))
                ],
                uniforms: &[
                    Uniform::new("color", UniformType::Vector(NumberType::Float, Dimension::D3)),
                    Uniform::new("radius", UniformType::Vector(NumberType::Float, Dimension::D2)),
                ],
                vertex_shader: "
            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
                passPosition = position;
            }",
                fragment_shader: "
            void main() {
                if (
                    passPosition.x >= 0.5 - radius.x && passPosition.x <= 0.5 + radius.x &&
                    passPosition.y >= 0.5 - radius.y && passPosition.y <= 0.5 + radius.y
                ) {
                    gl_FragColor = vec4(color, 1.0);
                } else {
                    discard;
                }
            }",
            };

            let mut shader = ShaderProgram::new(golem, shader_description)?;
            let mut vertex_buffer = VertexBuffer::new(golem)?;
            let mut element_buffer = ElementBuffer::new(golem)?;
            vertex_buffer.set_data(&quad_vertices);
            element_buffer.set_data(&quad_indices);

            shader.bind();
            let color = match if_hovering {
                true => self.hover_color,
                false => self.base_color
            };
            shader.set_uniform("color", UniformValue::Vector3([
                color.get_red_float(), color.get_green_float(), color.get_blue_float()
            ]));
            shader.set_uniform("radius", UniformValue::Vector2([
                used_width * 0.5, used_height * 0.5
            ]));
            unsafe {
                shader.draw(
                    &vertex_buffer, &element_buffer,
                    0..quad_indices.len(), GeometryMode::Triangles
                )?;
            }
        }

        Ok(RenderResultStruct {
            drawn_region: Box::new(drawn_region),
            filter_mouse_actions: true
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
    // TODO Check the returned drawn region
}