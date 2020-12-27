use golem::*;
use knukki::*;

fn main() {
    let mut menu = SimpleFlatMenu::new(Some(Color::rgb(100, 0, 0)));
    menu.add_component(
        Box::new(TestComponent { red: 100, green: 0 }),
        ComponentDomain::between(0.1, 0.1, 0.7, 0.3),
    );
    menu.add_component(
        Box::new(TestComponent {
            red: 100,
            green: 200,
        }),
        ComponentDomain::between(0.3, 0.5, 0.6, 0.9),
    );

    let app = Application::new(Box::new(menu));
    start(app, "Click the colors");
}

struct TestComponent {
    red: u8,
    green: u8,
}

impl Component for TestComponent {
    fn on_attach(&mut self, buddy: &mut dyn ComponentBuddy) {
        buddy.subscribe_mouse_click();
    }

    fn on_mouse_click(&mut self, _event: MouseClickEvent, buddy: &mut dyn ComponentBuddy) {
        self.red = self.red.wrapping_add(100);
        self.green = self.green.wrapping_add(17);
        buddy.request_render();
    }

    fn render(
        &mut self,
        golem: &Context,
        _region: RenderRegion,
        _buddy: &mut dyn ComponentBuddy,
        _force: bool
    ) -> RenderResult {
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
            fragment_input: &[],
            uniforms: &[
                Uniform::new("red", UniformType::Scalar(NumberType::Float)),
                Uniform::new("green", UniformType::Scalar(NumberType::Float)),
            ],
            vertex_shader: "
            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
            }",
            fragment_shader: "
            void main() {
                gl_FragColor = vec4(red, green, 1.0, 1.0);
            }",
        };

        let mut shader = ShaderProgram::new(golem, shader_description)?;
        let mut vertex_buffer = VertexBuffer::new(golem)?;
        let mut element_buffer = ElementBuffer::new(golem)?;
        vertex_buffer.set_data(&quad_vertices);
        element_buffer.set_data(&quad_indices);

        shader.bind();
        shader.set_uniform("red", UniformValue::Float(self.red as f32 / 255.0));
        shader.set_uniform("green", UniformValue::Float(self.green as f32 / 255.0));
        unsafe {
            shader.draw(
                &vertex_buffer, &element_buffer, 
                0..quad_indices.len(), GeometryMode::Triangles
            )?;
        }
        entire_render_result()
    }
}
