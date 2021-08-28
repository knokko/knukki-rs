use lazy_static::lazy_static;
use sha2::{Sha256, Digest, digest::Output};
use crate::Color;

/// The description of a *FragmentOnlyShader*. This description contains the source code of the
/// main method of the fragment shader and tells how many parameters/uniform variables it needs.
///
/// ## Usage
/// You can use a *FragmentOnlyShaderDescription* by wrapping it in a *FragmentOnlyShader* and
/// passing that to `Renderer.apply_fragment_shader`. The corresponding *FragmentOnlyDrawParameters*
/// should have a value for each variable declared in the description. For instance, if `num_colors`
/// is 2, `FragmentOnlyDrawParameters.colors` should be a slice of length 2.
///
/// ## (Uniform) variables
/// The *FragmentOnlyShaderDescription* also determines which uniform variables (or something
/// similar like push constants) the `Renderer` will inject in the shader. It will include the
/// following variables:
///
/// "matrix1", "matrix2", ..., "matrixN" where N = `num_float_matrices`
///
/// "color1", "color2", ..., "colorN" where N = `num_colors`
///
/// "floatVector1", "floatVector2", ..., "floatVectorN" where N = 'num_float_vectors`
///
/// "intVector1", "intVector2", ..., "intVectorN" where N = `num_int_vectors`
///
/// "float1", "float2", ..., "floatN" where N = `num_floats`
///
/// "int1", "int2", ..., "intN" where N = `num_ints`
///
/// ## Parameter system motivation
/// In case you're wondering why you can't just choose the names of the uniform variables yourself:
/// Switching and creating shaders are somewhat expensive operations. This parameter system forces
/// shaders to have the same parameter names, which allows the `Renderer` to 'combine' shaders to
/// improve performance. (But this is a future optimization idea; the current implementation
/// doesn't do this yet.)
pub struct FragmentOnlyShaderDescription {
    /// The source code of the **functions** of the fragment shader. This should **not** contain
    /// input, output, or uniform variable declarations! (The `Renderer` will take care of this.)
    pub source_code: String,
    /// The number of (float) mat4 uniform variables this shader needs. If you need smaller matrices,
    /// you can simply ignore some of the rows or columns.
    pub num_float_matrices: u8,
    pub num_colors: u8,
    pub num_float_vectors: u8,
    pub num_int_vectors: u8,
    pub num_floats: u8,
    pub num_ints: u8,
}

/// This struct wraps a *FragmentOnlyShaderDescription* and some implementation-dependant other
/// variables for the sake of optimization. This struct must be used in
/// `Renderer.apply_fragment_shader` rather than *FragmentOnlyShaderDescription* itself.
///
/// `Component`s are encouraged to create only 1 *FragmentOnlyShader* for each
/// *FragmentOnlyShaderDescription* (the best moment to construct a *FragmentOnlyShader* is normally
/// during the construction of the `Component`). This is expected to be more efficient than
/// creating a new *FragmentOnlyShader* each frame.
#[allow(dead_code)]
pub struct FragmentOnlyShader {
    pub(crate) description: FragmentOnlyShaderDescription,
    pub(crate) hash: Output<Sha256>
}

impl FragmentOnlyShader {
    pub fn new(description: FragmentOnlyShaderDescription) -> Self {
        let hash = Sha256::digest(description.source_code.as_bytes());
        Self {
            description,
            hash
        }
    }
}

/// The parameters that will be supplied to the corresponding *FragmentOnlyShader* during
/// `Renderer.apply_fragment_shader`. See the documentation of *FragmentOnlyShaderDescription* for
/// more information.
#[derive(Default)]
pub struct FragmentOnlyDrawParameters<'a> {
    pub float_matrices: &'a [[f32; 16]],
    pub colors: &'a [Color],
    pub float_vectors: &'a [[f32; 4]],
    pub int_vectors: &'a [[i32; 4]],
    pub floats: &'a [f32],
    pub ints: &'a [i32]
}

fn create_variable_names(prefix: &'static str) -> Vec<&'static str> {
    (0 ..= 255).into_iter().map(|counter| {
        Box::leak(Box::new(format!("{}{}", prefix, counter))) as &'static str
    }).collect()
}

lazy_static! {
    pub(crate) static ref MATRIX_VARIABLE_NAMES: Vec<&'static str> = create_variable_names("matrix");
    pub(crate) static ref COLOR_VARIABLE_NAMES: Vec<&'static str> = create_variable_names("color");
    pub(crate) static ref FLOAT_VECTOR_VARIABLE_NAMES: Vec<&'static str> = create_variable_names("floatVector");
    pub(crate) static ref INT_VECTOR_VARIABLE_NAMES: Vec<&'static str> = create_variable_names("intVector");
    pub(crate) static ref FLOAT_VARIABLE_NAMES: Vec<&'static str> = create_variable_names("float");
    pub(crate) static ref INT_VARIABLE_NAMES: Vec<&'static str> = create_variable_names("int");
}
