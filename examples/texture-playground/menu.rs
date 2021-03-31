use knukki::*;
use golem::*;
use std::num::NonZeroU32;

pub const EXAMPLE_NAME: &'static str = "texture-playground";

pub fn create_app() -> Application {
    let mut menu = SimpleFlatMenu::new(Some(Color::rgb(100, 200, 50)));

    menu.add_component(Box::new(TextureTestComponent::new()), ComponentDomain::between(0.1, 0.1, 0.5, 0.5));

    Application::new(Box::new(menu))
}

struct TextureTestComponent {

}

impl TextureTestComponent {
    fn new() -> Self {
        Self {}
    }
}

impl Component for TextureTestComponent {
    fn on_attach(&mut self, _buddy: &mut dyn ComponentBuddy) {
    }

    fn render(
        &mut self,
        renderer: &Renderer,
        _buddy: &mut dyn ComponentBuddy,
        _force: bool,
    ) -> RenderResult {
        renderer.clear(Color::rgb(200, 0, 0));

        let texture = renderer.load_texture(&create_image())?;
        texture.set_active(NonZeroU32::new(1).unwrap());

        let shader_id = ShaderId::from_strs("knukki", "Test.SimpleTexture");
        renderer.use_cached_shader(&shader_id, create_shader, |shader| {

            shader.set_uniform("image", UniformValue::Int(1))?;
            unsafe {
                shader.draw(
                    renderer.get_quad_vertices(),
                    renderer.get_quad_indices(),
                    0..renderer.get_num_quad_indices(),
                    GeometryMode::Triangles,
                )
            }
        })?;

        entire_render_result()
    }
}

#[rustfmt::skip]
fn create_shader(golem: &Context) -> Result<ShaderProgram, GolemError> {
    let description = ShaderDescription {
        vertex_input: &[
            Attribute::new("position", AttributeType::Vector(Dimension::D2))
        ],
        fragment_input: &[
            Attribute::new("passPosition", AttributeType::Vector(Dimension::D2))
        ],
        uniforms: &[
            Uniform::new("image", UniformType::Sampler2D),
        ],
        vertex_shader: "
            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
                passPosition = position;
            }",
        fragment_shader: "
            void main() {
                gl_FragColor = texture(image, passPosition.xy * 0.5 + vec2(0.5, 0.5));
            }",
    };

    ShaderProgram::new(golem, description)
}

fn create_image() -> knukki::Texture {
    let width = 8;
    let height = 8;
    let mut image = knukki::Texture::new(width, height, Color::rgb(0, 0, 0));

    for x in 0 .. width {
        for y in 0 .. height {
            image[x][y as usize] = Color::rgb((255 * x / width) as u8, (255 * y / height) as u8, 0);
        }
    }

    image
}