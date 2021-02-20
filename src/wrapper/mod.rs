#[cfg(not(target="wasm32-unknown-unknown"))]
mod desktop;
#[cfg(not(target="wasm32-unknown-unknown"))]
pub use desktop::*;

#[cfg(target="wasm32-unknown-unknown")]
mod web;
#[cfg(target="wasm32-unknown-unknown")]
pub use web::*;
