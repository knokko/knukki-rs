use crate::*;
use golem::*;
use std::cell::RefCell;

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
}

pub(super) struct GolemRenderStorage {
    // Frequently used and cheap buffers
    quad_vertices: VertexBuffer,
    quad_indices: ElementBuffer,
}

impl GolemRenderStorage {
    fn new(context: &Context) -> Result<Self, GolemError> {
        let mut quad_vertices = VertexBuffer::new(context)?;
        #[rustfmt::skip]
        quad_vertices.set_data(&[-1.0, -1.0,    1.0, -1.0,    1.0, 1.0,    -1.0, 1.0]);

        let mut quad_indices = ElementBuffer::new(context)?;
        quad_indices.set_data(&[0, 1, 2, 2, 3, 0]);

        Ok(Self {
            quad_vertices,
            quad_indices,
        })
    }
}

struct ShaderCache {}

struct CachedShader {
    last_used: u64,
    shader: ShaderProgram,
}
