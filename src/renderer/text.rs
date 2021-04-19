use crate::*;

use unicode_segmentation::UnicodeSegmentation;

use std::cell::RefCell;
use std::collections::{
    HashMap,
    HashSet,
};
use std::num::NonZeroU32;

pub struct TextRenderer {
    internal: RefCell<InternalTextRenderer>
}

impl TextRenderer {
    pub fn new() -> Self {
        Self { internal: RefCell::new(InternalTextRenderer::new()) }
    }

    pub fn register_font(&mut self, font: Box<dyn Font>) -> FontHandle {
        let mut internal = self.internal.borrow_mut();
        internal.register_font(font)
    }

    pub fn draw_text(
        &self,
        text: &str,
        font: FontHandle,
        position: TextDrawPosition,
        renderer: &Renderer,
    ) -> Result<DrawnTextPosition, TextRenderError> {
        let mut internal = self.internal.borrow_mut();
        internal.draw_text(text, font, position, renderer)
    }
}

struct InternalTextRenderer {
    fonts: HashMap<FontHandle, FontEntry>,
}

impl InternalTextRenderer {
    pub fn new() -> Self {
        Self { fonts: HashMap::new() }
    }

    pub fn register_font(&mut self, font: Box<dyn Font>) -> FontHandle {

        let handle = FontHandle { internal: self.fonts.len() as u16 };

        let atlas_group = TextureAtlasGroup::new(
            1024, 1024, 100, 10, 1, 1
        );

        let char_textures = HashMap::new();
        let string_models = HashMap::new();

        self.fonts.insert(handle, FontEntry { font, atlas_group, char_textures, string_models });
        handle
    }

    pub fn draw_text(
        &mut self,
        text: &str,
        font: FontHandle,
        position: TextDrawPosition,
        renderer: &Renderer,
    ) -> Result<DrawnTextPosition, TextRenderError> {
        if !self.fonts[&font].string_models.contains_key(text) {
            let text_model = self.create_text_model(
                #[cfg(feature = "golem_rendering")]
                renderer.get_context(),
                font,
                text
            )?;
            self.fonts.get_mut(&font).expect("Font handle is valid").string_models.insert(text.to_string(), text_model);
        }

        Ok(self.draw_text_model(text, font, position, renderer))
    }

    fn create_text_model(
        &mut self,
        #[cfg(feature = "golem_rendering")]
        ctx: &golem::Context,
        font: FontHandle,
        text: &str
    ) -> Result<TextModel, TextRenderError> {

        let entry = self.fonts.get_mut(&font).expect("font handle is invalid");

        // This seems to be a reasonable value. Perhaps, I could improve it later
        let point_size = 100.0;

        #[derive(Copy, Clone)]
        struct GraphemePosition {
            min_x: f32,
            min_y: f32,
            max_x: f32,
            max_y: f32,
            texture_id: GroupTextureID
        }

        // TODO Add multi-line support. NOTE: When going for multi-line, don't try to place too many
        // unique graphemes in 1 go on the texture atlas group because I didn't optimize groups for
        // such usage.
        let mut offset_x = 0;
        let grapheme_positions: Vec<_> = text.graphemes(true).filter_map(|grapheme| {

            let font = &entry.font;
            let mut atlas_group = &mut entry.atlas_group;
            let maybe_grapheme_texture_id = entry.char_textures.entry(grapheme.to_string()).or_insert_with(
                || {
                    let raw_grapheme_texture = font.draw_grapheme(grapheme, point_size);
                    if let Some(grapheme_texture) = raw_grapheme_texture {

                        let grapheme_texture_width = grapheme_texture.texture.get_width();
                        let grapheme_texture_height = grapheme_texture.texture.get_height();

                        let maybe_texture_id = atlas_group.add_texture(grapheme_texture.texture);
                        if let Ok(texture_id) = maybe_texture_id {
                            Some(GroupGraphemeTexture {
                                texture_id,
                                offset_y: grapheme_texture.offset_y,
                                width: grapheme_texture_width,
                                height: grapheme_texture_height,
                            })
                        } else {
                            // Edge case: very big character
                            None
                        }
                    } else {

                        // This is in case of a whitespace
                        None
                    }
                }
            );

            if let Some(group_grapheme_texture) = maybe_grapheme_texture_id {
                let position = GraphemePosition {
                    min_x: offset_x as f32,
                    min_y: group_grapheme_texture.offset_y as f32,
                    max_x: (offset_x + group_grapheme_texture.width) as f32,
                    max_y: (group_grapheme_texture.offset_y + group_grapheme_texture.height) as f32,
                    texture_id: group_grapheme_texture.texture_id
                };
                offset_x += group_grapheme_texture.width;
                Some(position)
            } else {
                offset_x += entry.font.get_whitespace_width(point_size) as u32;
                None
            }
        }).collect();

        let width = offset_x;

        // TODO Improve this for multi-line models
        let height = (entry.font.get_max_ascent(point_size) + entry.font.get_max_descent(point_size).ceil()) as u32;

        let group_texture_ids: Vec<_> = grapheme_positions.iter().map(
            |grapheme_position| grapheme_position.texture_id
        ).collect();

        let placements = entry.atlas_group.place_textures(&group_texture_ids);
        let mut text_vertices = Vec::with_capacity(placements.len());

        for index in 0 .. placements.len() {
            let placement = placements[index].clone();
            let position = grapheme_positions[index];

            text_vertices.push(TextVertex {
                min_x: position.min_x,
                min_y: position.min_y,
                max_x: position.max_x,
                max_y: position.max_y,
                placement
            });
        }

        Ok(TextModel {
            font,
            width,
            height,

            #[cfg(feature = "golem_rendering")]
            fragments: create_text_model_fragments(ctx, &text_vertices, width, height)?,

            vertices: text_vertices,
        })
    }

    #[rustfmt::skip]
    #[cfg(feature = "golem_rendering")]
    fn create_default_shader(golem: &golem::Context) -> Result<golem::ShaderProgram, golem::GolemError> {
        use golem::*;

        let description = ShaderDescription {
            vertex_input: &[
                Attribute::new("position", AttributeType::Vector(Dimension::D2)),
                Attribute::new("textureCoordinates", AttributeType::Vector(Dimension::D2)),
            ],
            fragment_input: &[
                Attribute::new("passTextureCoordinates", AttributeType::Vector(Dimension::D2)),
            ],
            uniforms: &[
                Uniform::new("offset", UniformType::Vector(NumberType::Float, Dimension::D2)),
                Uniform::new("scale", UniformType::Vector(NumberType::Float, Dimension::D2)),
                Uniform::new("backgroundColor", UniformType::Vector(NumberType::Float, Dimension::D3)),
                Uniform::new("textColor", UniformType::Vector(NumberType::Float, Dimension::D3)),
                Uniform::new("textureSampler", UniformType::Sampler2D),
            ],
            vertex_shader: "
            void main() {
                gl_Position = vec4(offset + scale * position, 0.0, 1.0);
                passTextureCoordinates = textureCoordinates;
                passTextureIndex = textureIndex;
            }",
            fragment_shader: "
            void main() {
                float intensity = texture(textureSampler, passTextureCoordinates).r;
                vec3 color3d = intensity * textColor + (1.0 - intensity) * backgroundColor;
                gl_FragColor = vec4(color3d, 1.0);
            }",
        };

        ShaderProgram::new(golem, description)
    }

    fn compute_text_position(
        &self, model: &TextModel, position: TextDrawPosition, viewport: RenderRegion
    ) -> (UniformTextDrawPosition, DrawnTextPosition) {

        let local_max_width = position.max_x - position.min_x;
        let local_max_height = position.max_y - position.min_y;

        // Exceeding the max scale would cause the text to be rendered outside the given bounds
        let max_scale_x = local_max_width / model.width as f32;
        let max_scale_y = local_max_height / model.height as f32;

        // The adapted scales take the viewport into account
        let adapted_scale_x = max_scale_y * viewport.get_aspect_ratio();
        let adapted_scale_y = max_scale_x / viewport.get_aspect_ratio();

        // The width of the text should be equal to the max width or the height of the text should
        // be equal to the max height (or both if the aspect ratio is perfect)
        let (scale_x, scale_y) = if adapted_scale_x <= max_scale_x {
            (adapted_scale_x, max_scale_y)
        } else {
            (max_scale_x, adapted_scale_y)
        };

        // The actual width and height that the drawn text will occupy
        let draw_width = scale_x * model.width as f32;
        let draw_height = scale_y * model.height as f32;

        let margin_x = local_max_width - draw_width;
        let margin_y = local_max_height - draw_height;

        let offset_x = match position.horizontal_alignment {
            Left => position.min_x,
            Center => position.min_x + margin_x / 2.0,
            Right => position.max_x - draw_width
        };

        let offset_y = match position.vertical_alignment {
            Bottom=> position.min_y,
            Center => position.min_y + margin_y / 2.0,
            Right => position.max_y - draw_height
        };

        let uniform_position = UniformTextDrawPosition {
            offset_x,
            offset_y,
            scale_x,
            scale_y
        };

        let drawn_position = DrawnTextPosition {
            min_x: offset_x,
            min_y: offset_y,
            max_x: offset_x + draw_width,
            max_y: offset_y + draw_height,
        };

        (uniform_position, drawn_position)
    }

    fn draw_text_model(
        &mut self, text: &str, font: FontHandle, position: TextDrawPosition, renderer: &Renderer
    ) -> DrawnTextPosition {
        let model = &self.fonts[&font].string_models[text];
        debug_assert!(model.is_still_valid());

        let (uniform_position, drawn_position) = self.compute_text_position(model, position, renderer.get_viewport());

        let mut my_fonts = &mut self.fonts;
        let font_entry = my_fonts.get_mut(&font).expect("Valid model font handle");
        let mut atlas_group = &mut font_entry.atlas_group;
        let model = &font_entry.string_models[text];

        #[cfg(feature = "golem_rendering")]
            {
                use golem::*;



                let shader_id = ShaderId::from_strs("knukki", "DefaultTextShader");
                renderer.use_cached_shader(&shader_id, Self::create_default_shader, |shader| {
                    shader.set_uniform("offset", UniformValue::Vector2([
                        uniform_position.offset_x, uniform_position.offset_y
                    ]))?;
                    shader.set_uniform("scale", UniformValue::Vector2([
                        uniform_position.scale_x, uniform_position.scale_y
                    ]))?;
                    shader.set_uniform("backgroundColor", UniformValue::Vector3([
                        0.0, 0.0, 1.0
                    ]))?;
                    shader.set_uniform("textColor", UniformValue::Vector3([
                        1.0, 1.0, 0.0
                    ]))?;
                    // TODO Perhaps, I shouldn't hardcode 1 like this
                    shader.set_uniform("textureSampler", UniformValue::Int(1))?;

                    for fragment in &model.fragments {
                        // TODO Bind its atlas texture
                        let gpu_texture = atlas_group.get_gpu_texture::<GolemError, _>(fragment.atlas_index, |texture| {
                            let mut golem_texture = Texture::new(renderer.get_context())?;
                            golem_texture.set_image(
                                Some(&texture.create_pixel_buffer()),
                                texture.get_width(),
                                texture.get_height(),
                                ColorFormat::RGBA
                            );
                            Ok(golem_texture)
                        })?;
                        gpu_texture.set_active(NonZeroU32::new(1).unwrap()); // TODO Perhaps don't hardcode 1
                        unsafe {
                            shader.draw(
                                &fragment.vertex_buffer,
                                &fragment.element_buffer,
                                0..fragment.element_buffer.size() / 4, // TODO Test that this is correct
                                GeometryMode::Triangles,
                            )?;
                        }
                    }
                    Ok(())
                });
            }
        drawn_position
    }
}

struct UniformTextDrawPosition {
    offset_x: f32,
    offset_y: f32,
    scale_x: f32,
    scale_y: f32,
}

pub struct DrawnTextPosition {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

pub struct TextDrawPosition {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
    pub horizontal_alignment: HorizontalTextAlignment,
    pub vertical_alignment: VerticalTextAlignment,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum VerticalTextAlignment {
    Bottom,
    Center,
    Top,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum HorizontalTextAlignment {
    Left,
    Center,
    Right
}

struct TextVertex {
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
    placement: GroupTexturePlacement,
}

struct TextModel {
    vertices: Vec<TextVertex>,
    font: FontHandle,
    width: u32,
    height: u32,

    #[cfg(feature = "golem_rendering")]
    fragments: Vec<TextModelFragment>,
}

#[cfg(feature = "golem_rendering")]
type TextRenderError = golem::GolemError;

#[cfg(not(feature = "golem_rendering"))]
type TextRenderError = ();

#[cfg(feature = "golem_rendering")]
fn create_text_model_fragments(
    ctx: &golem::Context,
    vertices: &[TextVertex],
    width: u32,
    height: u32
) -> Result<Vec<TextModelFragment>, TextRenderError> {
    let mut atlas_indices = HashSet::new();
    for vertex in vertices {
        atlas_indices.insert(vertex.placement.get_cpu_atlas_index());
    }

    let result_fragments: Vec<Result<_, TextRenderError>> = atlas_indices.into_iter().map(|atlas_index| {

        let num_vertices = vertices.iter().filter(
            |vertex| vertex.placement.get_cpu_atlas_index() == atlas_index
        ).count();

        let mut vertex_data = Vec::with_capacity(4 * 4 * num_vertices);
        for vertex in vertices.iter().filter(
            |vertex| vertex.placement.get_cpu_atlas_index() == atlas_index
        ) {

            let atlas_pos = vertex.placement.get_position();
            let min_tex_x = (atlas_pos.min_x as f32 + 0.5) / width as f32;
            let min_tex_y = (atlas_pos.min_y as f32 + 0.5) / height as f32;
            let max_tex_x = (atlas_pos.min_x as f32 + atlas_pos.width as f32 - 0.5) / width as f32;
            let max_tex_y = (atlas_pos.min_y as f32 + atlas_pos.height as f32 - 0.5) / height as f32;

            let coordinates = [
                (vertex.min_x, vertex.min_y, min_tex_x, min_tex_y),
                (vertex.max_x, vertex.min_y, max_tex_x, min_tex_y),
                (vertex.max_x, vertex.max_y, max_tex_x, max_tex_y),
                (vertex.min_x, vertex.max_y, min_tex_x, max_tex_y),
            ];

            for (pos_x, pos_y, tex_x, tex_y) in &coordinates {
                vertex_data.push(*pos_x);
                vertex_data.push(*pos_y);
                vertex_data.push(*tex_x);
                vertex_data.push(*tex_y);
            }
        }

        // TODO This could be optimized by only creating the element buffer for the largest group,
        // and using only the first N of its elements during drawing. That having said, having more
        // than 1 fragment should be uncommon anyway.
        let mut vertex_buffer = golem::VertexBuffer::new(ctx)?;
        vertex_buffer.set_data(&vertex_data);

        let mut element_data = Vec::with_capacity(6 * num_vertices);
        for index in 0 .. num_vertices {
            let vertex_offset = 4 * index as u32;
            element_data.push(vertex_offset);
            element_data.push(vertex_offset + 1);
            element_data.push(vertex_offset + 2);
            element_data.push(vertex_offset + 2);
            element_data.push(vertex_offset + 3);
            element_data.push(vertex_offset);
        }

        let mut element_buffer = golem::ElementBuffer::new(ctx)?;
        element_buffer.set_data(&element_data);

        Ok(TextModelFragment {
            atlas_index,
            vertex_buffer,
            element_buffer,
        })
    }).collect();

    let has_errors = result_fragments.iter().any(|result_fragment| result_fragment.is_err());
    if has_errors {
        for result_fragment in result_fragments {
            if let Err(the_error) = result_fragment {
                return Err(the_error);
            }
        }

        unreachable!()
    }

    Ok(result_fragments.into_iter().map(
        |result_fragment| result_fragment.unwrap()
    ).collect())
}

struct TextModelFragment {
    atlas_index: u16,

    #[cfg(feature = "golem_rendering")]
    vertex_buffer: golem::VertexBuffer,

    #[cfg(feature = "golem_rendering")]
    element_buffer: golem::ElementBuffer,
}

impl TextModel {
    fn is_still_valid(&self) -> bool {
        for vertex in &self.vertices {
            if !vertex.placement.is_still_valid() {
                return false;
            }
        }

        true
    }

    #[cfg(feature = "golem_rendering")]
    fn create_element_buffer(&self, ctx: &golem::Context) -> Result<golem::ElementBuffer, golem::GolemError> {
        let mut element_data = Vec::with_capacity(6 * self.vertices.len());
        for index in 0 .. self.vertices.len() {
            let vertex_offset = 4 * index as u32;
            element_data.push(vertex_offset);
            element_data.push(vertex_offset + 1);
            element_data.push(vertex_offset + 2);
            element_data.push(vertex_offset + 2);
            element_data.push(vertex_offset + 3);
            element_data.push(vertex_offset);
        }

        let mut element_buffer = golem::ElementBuffer::new(ctx)?;
        element_buffer.set_data(&element_data);
        Ok(element_buffer)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct FontHandle {
    internal: u16
}

struct GroupGraphemeTexture {
    texture_id: GroupTextureID,
    offset_y: u32,
    width: u32,
    height: u32,
}

#[cfg(feature = "golem_rendering")]
type GpuTexture = golem::Texture;

#[cfg(not(feature = "golem_rendering"))]
type GpuTexture = ();

struct FontEntry {
    font: Box<dyn Font>,
    char_textures: HashMap<String, Option<GroupGraphemeTexture>>,
    atlas_group: TextureAtlasGroup<GpuTexture>,
    string_models: HashMap<String, TextModel>,
}

struct GraphemeTextureEntry {
    texture_id: Option<GroupTextureID>,
    width: u32,
    offset_y: u32
}
