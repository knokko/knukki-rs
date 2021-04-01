use crate::*;

mod manager;
mod included;
#[cfg(not(target_arch = "wasm32"))]
mod system;
#[cfg(target_arch = "wasm32")]
mod web;

pub use manager::*;
pub use included::*;
#[cfg(not(target_arch = "wasm32"))]
pub use system::*;
#[cfg(target_arch = "wasm32")]
pub use web::*;

pub trait Font {
    /// Draws the given grapheme cluster using the given point size. If it is a whitespace
    /// character, this will return None.
    fn draw_grapheme(&self, grapheme: &str, point_size: f32) -> Option<Texture>;
}