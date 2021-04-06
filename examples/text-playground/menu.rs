use knukki::*;
use golem::*;
use std::num::NonZeroU32;
use std::collections::HashMap;
use unicode_segmentation::UnicodeSegmentation;

pub const EXAMPLE_NAME: &'static str = "text-playground";

pub fn create_app() -> Application {
    let mut menu = SimpleFlatMenu::new(Some(Color::rgb(100, 200, 50)));

    menu.add_component(Box::new(TextureTestComponent::new()), ComponentDomain::between(0.1, 0.1, 0.5, 0.5));

    Application::new(Box::new(menu))
}

struct TextureTestComponent {
    atlas: Option<TextureAtlas>,
    placements: Option<Vec<Option<PlacedCharacter>>>,
    whitespace_width: u32,
    height: u32,

    gpu_texture: Option<golem::Texture>,
}

const POINT_SIZE: f32 = 100.0;

impl TextureTestComponent {
    fn new() -> Self {
        let font = knukki::create_default_font();
        Self {
            atlas: None, placements: None,
            gpu_texture: None,
            whitespace_width: 0,
            height: (font.get_max_ascent(POINT_SIZE) + font.get_max_descent(POINT_SIZE)) as u32,
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
        let background_color = Color::rgb(200, 0, 0);
        renderer.clear(background_color);

        if self.atlas.is_none() {
            let font = knukki::create_default_font();
            let (atlas, placements) = create_image(&font);
            self.atlas = Some(atlas);
            self.placements = Some(placements);
            self.whitespace_width = font.get_whitespace_width(POINT_SIZE) as u32;
        }

        let atlas = self.atlas.as_ref().unwrap();
        let positions = self.placements.as_ref().unwrap();

        if self.gpu_texture.is_none() {
            self.gpu_texture = Some(renderer.load_texture(&atlas.get_texture())?);
        }
        let texture = self.gpu_texture.as_ref().unwrap();
        texture.set_active(NonZeroU32::new(1).unwrap());

        // Note that the capacities are inexact
        let mut vertices = Vec::with_capacity(positions.len() * 16);
        let mut indices = Vec::with_capacity(positions.len() * 6);

        let tex_x = |x: u32| (x as f32 + 0.5) / atlas.get_texture().get_width() as f32;
        let tex_y = |y: u32| (y as f32 + 0.5) / atlas.get_texture().get_height() as f32;

        let mut offset_x = 0;
        for maybe_position_info in positions {

            if let Some(position_info) = maybe_position_info {
                let position = position_info.position;

                let min_x = offset_x as f32;
                let min_y = position_info.offset_y as f32;
                let max_x = min_x + position.width as f32;
                let max_y = min_y + position.height as f32;
                let tex_min_x = tex_x(position.min_x);
                let tex_min_y = tex_y(position.min_y);
                let tex_max_x = tex_x(position.min_x + position.width - 1);
                let tex_max_y = tex_y(position.min_y + position.height - 1);

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
            } else {
                offset_x += self.whitespace_width;
            }
        }

        let mut vertex_buffer = VertexBuffer::new(renderer.get_context())?;
        vertex_buffer.set_data(&vertices);
        let mut index_buffer = ElementBuffer::new(renderer.get_context())?;
        index_buffer.set_data(&indices);

        let shader_id = ShaderId::from_strs("knukki", "Test.SimpleTexture");
        renderer.use_cached_shader(&shader_id, create_shader, |shader| {

            shader.set_uniform("image", UniformValue::Int(1))?;
            shader.set_uniform("offset", UniformValue::Vector2([-1.0, -1.0]))?;
            shader.set_uniform("backgroundColor", UniformValue::Vector3([
                background_color.get_red_float(),
                background_color.get_green_float(),
                background_color.get_blue_float()
            ]))?;
            shader.set_uniform("textColor", UniformValue::Vector3([1.0, 1.0, 1.0]))?;

            let width = offset_x as f32;
            let height = self.height as f32;
            let aspect_ratio = renderer.get_viewport().get_aspect_ratio();

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
            Uniform::new("backgroundColor", UniformType::Vector(NumberType::Float, Dimension::D3)),
            Uniform::new("textColor", UniformType::Vector(NumberType::Float, Dimension::D3)),
        ],
        vertex_shader: "
            void main() {
                gl_Position = vec4(offset + scale * position, 0.0, 1.0);
                passTextureCoordinates = textureCoordinates;
            }",
        fragment_shader: "
            void main() {
                float intensity = texture(image, passTextureCoordinates).r;
                vec3 color3d = intensity * textColor + (1.0 - intensity) * backgroundColor;
                gl_FragColor = vec4(color3d, 1.0);
            }",
    };

    ShaderProgram::new(golem, description)
}

struct PlacedCharacter {
    position: TextureAtlasPosition,
    offset_y: u32,
}

fn create_image(font: &dyn knukki::Font) -> (TextureAtlas, Vec<Option<PlacedCharacter>>) {
    let the_string = "A̘ji nǗx?̘\r\n\0";

    struct GraphemeValue {
        index: usize,
        char_texture: CharTexture,
    }

    let mut grapheme_map = HashMap::new();
    for grapheme in the_string.graphemes(true) {
        if !grapheme_map.contains_key(grapheme) {
            let index = grapheme_map.len();
            let maybe_char_texture = font.draw_grapheme(grapheme, POINT_SIZE);
            if let Some(char_texture) = maybe_char_texture {

                // Avoid including whitespace textures (that would have a very small width and/or height)
                if char_texture.texture.get_width() > 2 && char_texture.texture.get_height() > 2 {
                    grapheme_map.insert(grapheme, GraphemeValue { index, char_texture });
                }
            }
        }
    }

    let mut texture_vec = vec![None; grapheme_map.len()];
    for (_grapheme, value) in &grapheme_map {
        texture_vec[value.index] = Some(&value.char_texture.texture);
    }
    let texture_vec: Vec<_> = texture_vec.into_iter().map(|maybe_texture| maybe_texture.unwrap()).collect();

    let mut atlas = TextureAtlas::new(1024, 1024);
    let placement_info = atlas.add_textures(&texture_vec, false);

    // Note that the capacity is just an estimation
    let mut result_vec = Vec::with_capacity(grapheme_map.len());
    for grapheme in the_string.graphemes(true) {
        let maybe_value = grapheme_map.get(grapheme);
        if let Some(value) = maybe_value {
            let position = placement_info.placements[value.index].get_position().unwrap();
            result_vec.push(Some(PlacedCharacter {
                position,
                offset_y: value.char_texture.offset_y
            }));
        } else {
            result_vec.push(None);
        }
    }

    (atlas, result_vec)
}
