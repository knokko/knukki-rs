include!("../menu.rs");

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn web_start() {
    start(create_app(), "Hover Color Circle Menu")
}