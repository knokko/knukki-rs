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
    MouseEvent,
    WebGlRenderingContext,
    Window,
    window
};
use wasm_bindgen::__rt::core::cell::RefCell;

pub fn start(app: Application, title: &str) {
    let canvas = create_canvas();

    // Any event can cause any component to request a redraw, so this must be shared between all
    // event handlers.
    let force_next_render = Rc::new(Cell::new(true));

    // Similarly, all event handlers must have access to the application
    let wrap_app = Rc::new(RefCell::new(app));

    maintain_canvas_size(&canvas, Rc::clone(&force_next_render));
}

fn create_canvas() -> HtmlCanvasElement {
    let the_window = window().expect("Expected window");
    let document: Document = the_window.document().expect("Expected document");
    let canvas_element: Element = document.create_element("canvas").expect("Expected can create canvas");
    let canvas: HtmlCanvasElement = canvas_element.dyn_into::<HtmlCanvasElement>().expect("Expected canvas to be a canvas");

    canvas.set_id("knukki-canvas");

    canvas.set_width(get_window_width());
    canvas.set_height(get_window_height());

    document.append_with_node_1(&canvas).expect("Should be able to insert the knukki canvas");

    canvas
}

fn propagate_mouse_events(
    canvas: &HtmlCanvasElement,
    wrap_app: &Rc<RefCell<Application>>,
    force_next_render: Rc<Cell<bool>>
) {
    let the_window = window().expect("Expected a window");

    // Note: this is a clone of a JS reference to the canvas; not a clone of the canvas
    let canvas = canvas.clone();

    fn get_x(event: &MouseEvent) -> f32 {
        let int_x = event.client_x() as f32;
        int_x / get_window_width() as f32
    }

    fn get_y(event: &MouseEvent) -> f32 {
        let int_y = event.client_y() as f32;
        int_y / get_window_height() as f32
    }

    fn get_button(event: &MouseEvent) -> MouseButton {
        let js_button = event.button() as u8;

        // The knukki MouseButton conventions state that 1 should be right mouse and 2 should be
        // mouse wheel. In the JS MouseEvent, this is the other way around.
        let knukki_button = match js_button {
            1 => 2,
            2 => 1,
            _ => js_button
        };
        MouseButton::new(knukki_button)
    }

    // This mouse will be associated with the standard DOM events. I might add support for
    // controllers or keyboard-controlled mouses later.
    let primary_mouse = Mouse::new(0);

    let click_wrap_app = Rc::clone(wrap_app);
    let click_closure = Closure::wrap(Box::new(move |event| {
        let mut app = click_wrap_app.borrow_mut();
        app.fire_mouse_click_event(MouseClickEvent::new(
            primary_mouse,
            Point::new(get_x(&event), get_y(&event)),
            get_button(&event)
        ));
    }) as Box<dyn FnMut(MouseEvent)>);

    the_window.add_event_listener_with_callback("click", click_closure.as_ref().unchecked_ref())
        .expect("Should be able to add click listener");

    click_closure.forget();
}

fn maintain_canvas_size(canvas: &HtmlCanvasElement, force_next_render: Rc<Cell<bool>>) {
    let the_window = window().expect("Expected a window");

    // Note: This is a clone of a reference to the JS canvas; not a clone of the actual canvas
    let canvas = canvas.clone();

    let resize_closure = Closure::wrap(Box::new(move || {
        canvas.set_width(get_window_width());
        canvas.set_height(get_window_height());
        force_next_render.set(true);
        // TODO Fire resize event
    }) as Box<dyn FnMut()>);

    the_window.add_event_listener_with_callback(
        "resize", resize_closure.as_ref().unchecked_ref()
    ).expect("Should be able to add resize listener");

    resize_closure.forget();
}

fn get_window_width() -> u32 {
    extract_window_size("innerWidth", window().expect("Expected a window").inner_width())
}

fn get_window_height() -> u32 {
    extract_window_size("innerHeight", window().expect("Expected a window").inner_height())
}

fn extract_window_size<E: Debug>(name: &'static str, width_or_height: Result<JsValue, E>) -> u32 {
    width_or_height.expect(&*("Window.".to_owned() + name + " should exist"))
        .as_f64().expect(&*("Window.".to_owned() + name + " should be a JS number")) as u32
}