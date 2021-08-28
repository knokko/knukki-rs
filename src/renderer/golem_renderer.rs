use crate::*;
use golem::*;
use std::cell::RefCell;
use std::collections::HashMap;

impl Renderer {
    /// Constructs a new `Renderer` that will draw onto the given golem `Context` within the given
    /// *initial_viewport*. Normally, only the *wrapper* should use this function.
    pub fn new(context: Context, initial_viewport: RenderRegion) -> Self {
        Self {
            storage: GolemRenderStorage::new(&context).expect("Should be able to init storage"),
            context,
            text_renderer: TextRenderer::new(),
            viewport_stack: RefCell::new(vec![initial_viewport]),
            scissor_stack: RefCell::new(vec![initial_viewport]),
        }
    }

    /// Sets the color of all pixels within the current viewport and scissor to the given `Color`.
    pub fn clear(&self, color: Color) {
        self.context.set_clear_color(
            color.get_red_float(),
            color.get_green_float(),
            color.get_blue_float(),
            color.get_alpha_float(),
        );
        self.context.clear();
    }

    /// Uses the given *FragmentOnlyShader* to fill the rectangular region defined by *min_x*,
    /// *min_y*, *max_x*, and *max_y* (each of them should be between 0.0 and 1.0) using the given
    /// *parameters* (typically uniform variables). If you don't want to draw on the entire
    /// rectangular region, you can let the fragment shader *discard* those pixels.
    pub fn apply_fragment_shader(
        &self, min_x: f32, min_y: f32, max_x: f32, max_y: f32,
        shader: &FragmentOnlyShader, parameters: FragmentOnlyDrawParameters
    ) {
        let shader_name = format!("FragmentOnlyShader {:?}", shader.hash.as_slice());
        self.use_cached_shader(
            &ShaderId::from_strings("knukki".to_string(), shader_name),
            |golem| {

                let mut uniforms = Vec::new();
                uniforms.push(Uniform::new(
                    "vertexBounds",
                    UniformType::Vector(NumberType::Float, Dimension::D4)
                ));
                for matrix_counter in 1 ..= shader.description.num_float_matrices {
                    uniforms.push(Uniform::new(
                        MATRIX_VARIABLE_NAMES[matrix_counter as usize],
                        UniformType::Matrix(Dimension::D4)
                    ));
                }
                for color_counter in 1 ..= shader.description.num_colors {
                    uniforms.push(Uniform::new(
                        COLOR_VARIABLE_NAMES[color_counter as usize],
                        UniformType::Vector(NumberType::Float, Dimension::D4)
                    ));
                }
                for vector_counter in 1 ..= shader.description.num_float_vectors {
                    uniforms.push(Uniform::new(
                        FLOAT_VECTOR_VARIABLE_NAMES[vector_counter as usize],
                        UniformType::Vector(NumberType::Float, Dimension::D4)
                    ));
                }
                for vector_counter in 1 ..= shader.description.num_int_vectors {
                    uniforms.push(Uniform::new(
                        INT_VECTOR_VARIABLE_NAMES[vector_counter as usize],
                        UniformType::Vector(NumberType::Int, Dimension::D4)
                    ));
                }
                for float_counter in 1 ..= shader.description.num_floats {
                    uniforms.push(Uniform::new(
                        FLOAT_VARIABLE_NAMES[float_counter as usize],
                        UniformType::Scalar(NumberType::Float)
                    ));
                }
                for int_counter in 1 ..= shader.description.num_ints {
                    uniforms.push(Uniform::new(
                        INT_VARIABLE_NAMES[int_counter as usize],
                        UniformType::Scalar(NumberType::Int)
                    ));
                }

                let shader_description = ShaderDescription {
                    vertex_input: &[
                        Attribute::new("vertexInnerPosition", AttributeType::Vector(Dimension::D2))
                    ],
                    fragment_input: &[
                        Attribute::new("innerPosition", AttributeType::Vector(Dimension::D2)),
                        Attribute::new("outerPosition", AttributeType::Vector(Dimension::D2))
                    ],
                    uniforms: &uniforms,
                    vertex_shader: "
                void main() {
                    innerPosition = 0.5 * vertexInnerPosition + 0.5;
                    vec2 bottomLeftBounds = vertexBounds.xy;
                    vec2 topRightBounds = vertexBounds.zw;
                    outerPosition = bottomLeftBounds + innerPosition * (topRightBounds - bottomLeftBounds);
                    gl_Position = vec4(2.0 * outerPosition - vec2(1.0, 1.0), 0.0, 1.0);
                }
            ",
                    fragment_shader: &shader.description.source_code
                };
                ShaderProgram::new(golem, shader_description)
            }, |shader_program| {
                shader_program.set_uniform("vertexBounds", UniformValue::Vector4([min_x, min_y, max_x, max_y]))?;
                for matrix_counter in 1 ..= shader.description.num_float_matrices {
                    let _result = shader_program.set_uniform(
                        &format!("matrix{}", matrix_counter),
                        UniformValue::Matrix4(parameters.float_matrices[matrix_counter as usize - 1])
                    );
                }
                for color_counter in 1 ..= shader.description.num_colors {
                    let _result = shader_program.set_uniform(
                        &format!("color{}", color_counter),
                        UniformValue::Vector4(parameters.colors[color_counter as usize - 1].to_float_array())
                    );
                }
                for vector_counter in 1 ..= shader.description.num_float_vectors {
                    let _result = shader_program.set_uniform(
                        &format!("floatVector{}", vector_counter),
                        UniformValue::Vector4(parameters.float_vectors[vector_counter as usize - 1])
                    );
                }
                for vector_counter in 1 ..= shader.description.num_int_vectors {
                    let _result = shader_program.set_uniform(
                        &format!("intVector{}", vector_counter),
                        UniformValue::IVector4(parameters.int_vectors[vector_counter as usize - 1])
                    );
                }
                for float_counter in 1 ..= shader.description.num_floats {
                    let _result = shader_program.set_uniform(
                        &format!("float{}", float_counter),
                        UniformValue::Float(parameters.floats[float_counter as usize - 1])
                    );
                }
                for int_counter in 1 ..= shader.description.num_ints {
                    let _result = shader_program.set_uniform(
                        &format!("int{}", int_counter),
                        UniformValue::Int(parameters.ints[int_counter as usize - 1])
                    );
                }

                unsafe {
                    shader_program.draw(
                        self.get_quad_vertices(),
                        self.get_quad_indices(),
                        0 .. self.get_num_quad_indices(),
                        GeometryMode::Triangles
                    )
                }
            }
        ).expect("Shader shouldn't fail");
    }

    /// Gets the golem `Context` of this `Renderer`. Use this context to perform drawing operations
    /// that are not covered by the other methods of `Renderer`. Note that using this will damage
    /// the portability of the application since this will only work when a Golem renderer is used.
    pub fn get_context(&self) -> &Context {
        &self.context
    }

    // This will be handled internally.
    pub(super) fn apply_viewport_and_scissor(&self) {
        self.get_viewport().set_viewport(&self.context);
        self.get_scissor().set_scissor(&self.context);
    }

    /// Gets a reference to a `VertexBuffer` representing the basic `quad` model (simply the
    /// positions [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] ).
    ///
    /// This model can be surprisingly useful for `Component`s because this simple model can be
    /// quite powerful in combination with the right (fragment) shader: by discarding the right
    /// pixels, it is easy to construct other shapes like circles. It is also great for drawing
    /// basic images.
    ///
    /// As explained above, it can be useful for many `Component`. It would be a slight waste of
    /// resources to let every component create its own quad `VertexBuffer`. To solve this issue,
    /// all components in need of the quad model can simply share this one.
    pub fn get_quad_vertices(&self) -> &VertexBuffer {
        &self.storage.quad_vertices
    }

    /// Gets a reference to the corresponding `ElementBuffer` of the `VertexBuffer` given by the
    /// `get_quad_vertices` method. (These indices are just [(0, 1, 2), (2, 3, 0)].)
    pub fn get_quad_indices(&self) -> &ElementBuffer {
        &self.storage.quad_indices
    }

    /// Gets the number of indices in the `ElementBuffer` given by the `get_quad_indices`
    /// method, in integers (which is just 6).
    pub fn get_num_quad_indices(&self) -> usize {
        6
    }

    /// Checks if the shader with the given *id* has been cached by this `Renderer`. If so, `bind`s
    /// that shader and calls the given *use_shader* closure.
    ///
    /// If the shader with the given *id* is **not** found in the cache, the given `create_shader`
    /// closure will be called to create this. Then, it will be stored in the cache and its `bind`
    /// function will be called. And finally, the given *use_shader* closure will be called.
    ///
    /// ## Motivation
    /// Caching shaders can make the implementation of the `render` methods of `Component`s easier
    /// while also improving performance: `Component`s often need shader(s) for rendering, and they
    /// either need to create it at the start of every call of their `render` method (which is very
    /// bad for performance). Or, they could create it lazily during their first `render` call and
    /// store it for later (which is annoying to program because it requires adding an extra
    /// `Option<ShaderProgram>` field and maintain that). That would be better for performance, but
    /// is still suboptimal because every `Component` will need its **own** instance of the
    /// shader it need(s), even if many other `Component`s need that exact same shader.
    ///
    /// When `Component`s use this method, they no longer need to worry about storing the shader
    /// (because the `Renderer` will take care of that), and it will automatically be shared by all
    /// other `Component` that use this method and the same shader **id**.
    pub fn use_cached_shader(
        &self,
        id: &ShaderId,
        create_shader: impl FnOnce(&golem::Context) -> Result<ShaderProgram, GolemError>,
        use_shader: impl FnOnce(&mut ShaderProgram) -> Result<(), GolemError>,
    ) -> Result<(), GolemError> {
        let mut cache = self.storage.shader_cache.borrow_mut();
        cache.use_shader(id, || create_shader(&self.context), use_shader)
    }

    pub fn load_texture(&self, cpu_texture: &crate::Texture) -> Result<golem::Texture, GolemError> {
        let mut gpu_texture = golem::Texture::new(&self.context)?;
        let pixel_buffer = cpu_texture.create_pixel_buffer();

        gpu_texture.set_image(
            Some(&pixel_buffer),
            cpu_texture.get_width(),
            cpu_texture.get_height(),
            ColorFormat::RGBA,
        );
        gpu_texture.set_wrap_h(TextureWrap::ClampToEdge)?;
        gpu_texture.set_wrap_v(TextureWrap::ClampToEdge)?;
        gpu_texture.set_magnification(TextureFilter::Linear)?;
        gpu_texture.set_minification(TextureFilter::Linear)?;

        Ok(gpu_texture)
    }
}

pub(super) struct GolemRenderStorage {
    // Frequently used and cheap buffers
    quad_vertices: VertexBuffer,
    quad_indices: ElementBuffer,

    shader_cache: RefCell<ShaderCache>,
}

impl GolemRenderStorage {
    fn new(context: &Context) -> Result<Self, GolemError> {
        let mut quad_vertices = VertexBuffer::new(context)?;
        #[rustfmt::skip]
        quad_vertices.set_data(&[-1.0, -1.0,    1.0, -1.0,    1.0, 1.0,    -1.0, 1.0]);

        let mut quad_indices = ElementBuffer::new(context)?;
        quad_indices.set_data(&[0, 1, 2, 2, 3, 0]);

        // Practice will have to tell whether 200 is good.
        let max_cached_shaders = 200;

        Ok(Self {
            quad_vertices,
            quad_indices,
            shader_cache: RefCell::new(ShaderCache::new(max_cached_shaders)),
        })
    }
}

struct ShaderCache {
    map: HashMap<ShaderId, CachedShader>,
    max_cached_shaders: usize,
    current_time: u64,
}

impl ShaderCache {
    fn new(max_cached_shaders: usize) -> Self {
        assert!(max_cached_shaders > 0);
        Self {
            map: HashMap::new(),
            max_cached_shaders,
            current_time: 0,
        }
    }

    fn get_existing(&mut self, id: &ShaderId) -> &mut ShaderProgram {
        let cached = self.map.get_mut(id).unwrap();
        cached.last_used = self.current_time;
        return &mut cached.shader;
    }

    fn use_shader(
        &mut self,
        id: &ShaderId,
        create_shader: impl FnOnce() -> Result<ShaderProgram, GolemError>,
        use_shader: impl FnOnce(&mut ShaderProgram) -> Result<(), GolemError>,
    ) -> Result<(), GolemError> {
        self.current_time += 1;

        // If we have the value already, update its last_used and return it
        // Unfortunately, we do 2 hash lookups. I tried using only 1, but couldn't convince compiler
        let has_already = self.map.contains_key(id);
        if has_already {
            let shader = self.get_existing(id);
            shader.bind();
            return use_shader(shader);
        }

        // If we reach this line, we didn't have the shader yet
        let new_length = self.map.len() + 1;

        // If we would exceed the maximum number of cached shaders, we remove the least recently used half
        if new_length > self.max_cached_shaders {
            let mut last_used_times: Vec<u64> = self
                .map
                .values()
                .map(|cached_shader| cached_shader.last_used)
                .collect();
            last_used_times.sort();
            let median = last_used_times[last_used_times.len() / 2];

            // Remove at least half of the cached shaders
            self.map
                .retain(|_id, cached_shader| cached_shader.last_used > median);
        }

        // Now that we are sure we won't exceed the maximum number of shaders, we can insert the
        // new shader, and return a reference to it.
        let value = self.map.entry(id.clone()).or_insert(CachedShader {
            last_used: self.current_time,
            shader: create_shader()?,
        });
        value.shader.bind();
        use_shader(&mut value.shader)
    }
}

struct CachedShader {
    last_used: u64,
    shader: ShaderProgram,
}

/// Represents a unique identifier for a pair of a vertex shader and fragment shader. This struct
/// has a `crate_name` and a `shader_name`. This struct is used for the `use_cached_shader` method
/// of `Renderer` to identify shaders.
///
/// ## Create name
/// The `crate_name` should be the name of the crate that defines the corresponding shader.
///
/// ## Shader name
/// The `shader_name` should be used to distinguish shaders that are defined by the same crate. All
/// shaders defined by the same crate must have a distinct `shader_name`.
#[derive(Eq, PartialEq, Hash, Clone)]
pub struct ShaderId {
    crate_name: String,
    shader_name: String,
}

impl ShaderId {
    /// Constructs a `ShaderId` with the given `crate_name` and `shader_name`. See the documentation
    /// of this struct for more information.
    pub fn from_strings(crate_name: String, shader_name: String) -> Self {
        Self {
            crate_name,
            shader_name,
        }
    }

    /// Constructs a `ShaderId` with the given `crate_name` and `shader_name`. See the documentation
    /// of this struct for more information.
    pub fn from_strs(crate_name: &str, shader_name: &str) -> Self {
        Self {
            crate_name: crate_name.to_string(),
            shader_name: shader_name.to_string(),
        }
    }
}
