use crate::*;

use std::cell::Cell;
use std::fmt::Debug;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use web_sys::{
    Document,
    Element,
    HtmlCanvasElement,
    HtmlElement,
    WebGlRenderingContext,
    Window,
    window
};

pub fn start(mut app: Application, title: &str) {
    let canvas = create_canvas();

    // Any event can cause any component to request a redraw, so this must be shared between all
    // event handlers.
    let force_next_render = Rc::new(Cell::new(true));

    maintain_canvas_size(&canvas, Rc::clone(&force_next_render));
}

fn create_canvas() -> HtmlCanvasElement {
    let the_window = window().expect("Expected window");
    let document: Document = the_window.document().expect("Expected document");
    let canvas_element: Element = document.create_element("canvas").expect("Expected can create canvas");
    let canvas: HtmlCanvasElement = canvas_element.dyn_into::<HtmlCanvasElement>().expect("Expected canvas to be a canvas");

    canvas.set_id("knukki-canvas");

    canvas.set_width(get_window_width(&the_window));
    canvas.set_height(get_window_height(&the_window));

    document.append_with_node_1(&canvas).expect("Should be able to insert the knukki canvas");

    canvas
}

fn maintain_canvas_size(canvas: &HtmlCanvasElement, force_next_render: Rc<Cell<bool>>) {
    let the_window = window().expect("Expected a window");

    // Note: These are clones of references to JS objects rather than proper clones
    let canvas = canvas.clone();
    let window = the_window.clone();

    let resize_closure = Closure::wrap(Box::new(move || {
        canvas.set_width(get_window_width(&window));
        canvas.set_height(get_window_height(&window));
        force_next_render.set(true);
        // TODO Fire resize event
    }) as Box<dyn FnMut()>);

    the_window.add_event_listener_with_callback(
        "resize", resize_closure.as_ref().unchecked_ref()
    ).expect("Should be able to add resize listener");

    resize_closure.forget();
}

fn get_window_width(window: &Window) -> u32 {
    extract_window_size("innerWidth", window.inner_width())
}

fn get_window_height(window: &Window) -> u32 {
    extract_window_size("innerHeight", window.inner_height())
}

fn extract_window_size<E: Debug>(name: &'static str, width_or_height: Result<JsValue, E>) -> u32 {
    width_or_height.expect(&*("Window.".to_owned() + name + " should exist"))
        .as_f64().expect(&*("Window.".to_owned() + name + " should be a JS number")) as u32
}