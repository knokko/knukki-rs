use crate::*;
use golem::*;
use std::cell::RefCell;
use std::collections::HashMap;

impl Renderer {
    pub fn new(context: Context, initial_viewport: RenderRegion) -> Self {
        Self {
            storage: GolemRenderStorage::new(&context).expect("Should be able to init storage"),
            context,
            viewport_stack: RefCell::new(vec![initial_viewport]),
            scissor_stack: RefCell::new(vec![initial_viewport]),
        }
    }

    pub fn clear(&self, color: Color) {
        self.context.set_clear_color(
            color.get_red_float(),
            color.get_green_float(),
            color.get_blue_float(),
            color.get_alpha_float(),
        );
        self.context.clear();
    }

    pub fn get_context(&self) -> &Context {
        &self.context
    }

    pub fn apply_viewport_and_scissor(&self) {
        self.get_viewport().set_viewport(&self.context);
        self.get_scissor().set_scissor(&self.context);
    }

    pub fn get_quad_vertices(&self) -> &VertexBuffer {
        &self.storage.quad_vertices
    }

    pub fn get_quad_indices(&self) -> &ElementBuffer {
        &self.storage.quad_indices
    }

    pub fn get_num_quad_indices(&self) -> usize {
        6
    }

    pub fn use_shader(
        &self, id: &ShaderId,
        create_shader: impl FnOnce(&golem::Context) -> Result<ShaderProgram, GolemError>,
        use_shader: impl FnOnce(&mut ShaderProgram) -> Result<(), GolemError>
    ) -> Result<(), GolemError> {
        let mut cache = self.storage.shader_cache.borrow_mut();
        cache.use_shader(id, || create_shader(&self.context), use_shader)
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

    pub fn use_shader(
        &mut self, id: &ShaderId,
        create_shader: impl FnOnce() -> Result<ShaderProgram, GolemError>,
        use_shader: impl FnOnce(&mut ShaderProgram) -> Result<(), GolemError>
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
            let mut last_used_times: Vec<u64> = self.map.values().map(|cached_shader| cached_shader.last_used).collect();
            last_used_times.sort();
            let median = last_used_times[last_used_times.len() / 2];

            // Remove at least half of the cached shaders
            self.map.retain(|_id, cached_shader| cached_shader.last_used > median);
        }

        // Now that we are sure we won't exceed the maximum number of shaders, we can insert the
        // new shader, and return a reference to it.
        let value = self.map.entry(id.clone()).or_insert(CachedShader {
            last_used: self.current_time,
            shader: create_shader()?
        });
        value.shader.bind();
        use_shader(&mut value.shader)
    }
}

struct CachedShader {
    last_used: u64,
    shader: ShaderProgram,
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct ShaderId {
    crate_name: String,
    shader_name: String,
}

impl ShaderId {
    pub fn from_strings(crate_name: String, shader_name: String) -> Self {
        Self { crate_name, shader_name }
    }

    pub fn from_strs(crate_name: &str, shader_name: &str) -> Self {
        Self {
            crate_name: crate_name.to_string(),
            shader_name: shader_name.to_string(),
        }
    }
}
