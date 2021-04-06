use knukki::*;
use golem::*;
use std::num::NonZeroU32;
use unicode_segmentation::UnicodeSegmentation;

pub const EXAMPLE_NAME: &'static str = "text-playground";

pub fn create_app() -> Application {
    let mut menu = SimpleFlatMenu::new(Some(Color::rgb(100, 200, 50)));

    menu.add_component(Box::new(TextureTestComponent::new()), ComponentDomain::between(0.1, 0.1, 0.5, 0.5));

    Application::new(Box::new(menu))
}

struct TextureTestComponent {
    atlas: TextureAtlas,
    placements: TexturePlaceResult,

    gpu_texture: Option<golem::Texture>,
}

impl TextureTestComponent {
    fn new() -> Self {
        let font = knukki::create_default_font();
        let (atlas, placements) = create_image(&font);
        Self {
            atlas, placements,
            gpu_texture: None,
        }
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

        let atlas = &self.atlas;
        let positions = &self.placements;

        if self.gpu_texture.is_none() {
            self.gpu_texture = Some(renderer.load_texture(&atlas.get_texture())?);
        }
        let texture = self.gpu_texture.as_ref().unwrap();
        texture.set_active(NonZeroU32::new(1).unwrap());

        let mut vertices = Vec::with_capacity(positions.placements.len() * 16);
        let mut indices = Vec::with_capacity(positions.placements.len() * 6);

        let tex_x = |x: u32| x as f32 / atlas.get_texture().get_width() as f32;
        let tex_y = |y: u32| y as f32 / atlas.get_texture().get_height() as f32;

        let mut offset_x = 0;
        let mut height = 0;

        for position_info in &positions.placements {
            let position = position_info.get_position().unwrap();

            let min_x = offset_x as f32;
            let min_y = 0.0;
            let max_x = min_x + position.width as f32;
            let max_y = position.height as f32;
            let tex_min_x = tex_x(position.min_x);
            let tex_min_y = tex_y(position.min_y);
            let tex_max_x = tex_x(position.min_x + position.width);
            let tex_max_y = tex_y(position.min_y + position.height);

            let base_index = vertices.len() as u32 / 4;

            // Bottom-left
            vertices.push(min_x);
            vertices.push(min_y);
            vertices.push(tex_min_x);
            vertices.push(tex_min_y);

            // Bottom-right
            vertices.push(max_x);
            vertices.push(min_y);
            vertices.push(tex_max_x);
            vertices.push(tex_min_y);

            // Top-right
            vertices.push(max_x);
            vertices.push(max_y);
            vertices.push(tex_max_x);
            vertices.push(tex_max_y);

            // Top-left
            vertices.push(min_x);
            vertices.push(max_y);
            vertices.push(tex_min_x);
            vertices.push(tex_max_y);

            // Indices
            indices.push(base_index);
            indices.push(base_index + 1);
            indices.push(base_index + 2);

            indices.push(base_index + 2);
            indices.push(base_index + 3);
            indices.push(base_index);

            // Finalizing
            offset_x += position.width;
            height = height.max(position.height);
        }

        let mut vertex_buffer = VertexBuffer::new(renderer.get_context())?;
        vertex_buffer.set_data(&vertices);
        let mut index_buffer = ElementBuffer::new(renderer.get_context())?;
        index_buffer.set_data(&indices);

        let shader_id = ShaderId::from_strs("knukki", "Test.SimpleTexture");
        renderer.use_cached_shader(&shader_id, create_shader, |shader| {

            shader.set_uniform("image", UniformValue::Int(1))?;
            shader.set_uniform("offset", UniformValue::Vector2([-1.0, -1.0]))?;

            let width = offset_x as f32;
            let height = height as f32;
            let aspect_ratio = renderer.get_viewport().get_aspect_ratio();
            let aspect_ratio2 = width / height;

            let max_rel_scale_x = 2.0 / width;
            let max_rel_scale_y = 2.0 / height;

            let base_scale_x = 1.0;
            let base_scale_y = aspect_ratio;

            let pref_scale_x = max_rel_scale_x / base_scale_x;
            let pref_scale_y = max_rel_scale_y / base_scale_y;

            let pref_rel_scale = pref_scale_x.min(pref_scale_y);

            let scale_x = pref_rel_scale * base_scale_x;
            let scale_y = pref_rel_scale * base_scale_y;
            shader.set_uniform("scale", UniformValue::Vector2([scale_x, scale_y]))?;

            unsafe {
                shader.draw(
                    &vertex_buffer,
                    &index_buffer,
                    0..indices.len(),
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
            Attribute::new("position", AttributeType::Vector(Dimension::D2)),
            Attribute::new("textureCoordinates", AttributeType::Vector(Dimension::D2)),
        ],
        fragment_input: &[
            Attribute::new("passTextureCoordinates", AttributeType::Vector(Dimension::D2)),
        ],
        uniforms: &[
            Uniform::new("image", UniformType::Sampler2D),
            Uniform::new("offset", UniformType::Vector(NumberType::Float, Dimension::D2)),
            Uniform::new("scale", UniformType::Vector(NumberType::Float, Dimension::D2)),
        ],
        vertex_shader: "
            void main() {
                gl_Position = vec4(offset + scale * position, 0.0, 1.0);
                passTextureCoordinates = textureCoordinates;
            }",
        fragment_shader: "
            void main() {
                float intensity = texture(image, passTextureCoordinates).r;
                gl_FragColor = vec4(intensity, intensity, intensity, 1.0);
            }",
    };

    ShaderProgram::new(golem, description)
}

fn create_image(font: &dyn knukki::Font) -> (TextureAtlas, TexturePlaceResult) {
    let the_string = "Addnewrow?";
    let char_textures: Vec<_> = the_string.graphemes(true).map(
        |grapheme| font.draw_grapheme(grapheme, 100.0).unwrap()
    ).collect();
    let ref_char_textures: Vec<_> = char_textures.iter().map(|char_texture| char_texture).collect();

    let mut atlas = TextureAtlas::new(1024, 1024);
    let positions = atlas.add_textures(&ref_char_textures, false);

    (atlas, positions)
}
