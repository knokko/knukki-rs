use crate::*;

use serde::ser::{
    Serialize,
    Serializer,
    SerializeMap
};

use std::cell::{
    Cell,
    RefCell
};
use std::fmt::Debug;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::{
    JsCast,
    JsValue
};

use web_sys::{
    Document,
    Element,
    Event,
    HtmlCanvasElement,
    HtmlElement,
    MouseEvent,
    WebGlRenderingContext,
    window
};

pub fn start(app: Application, title: &str) {

    // For the sake of debugging, binding the console is the first thing that must be done
    bind_console();

    set_title(title);

    let canvas = create_canvas();

    // Any event can cause any component to request a redraw, so this must be shared between all
    // event handlers.
    let force_next_render = Rc::new(Cell::new(true));

    // Similarly, all event handlers must have access to the application
    let wrap_app = Rc::new(RefCell::new(app));

    maintain_canvas_size(&canvas, Rc::clone(&force_next_render));
    propagate_mouse_events(&wrap_app);
    start_render_loop(&canvas, wrap_app, force_next_render);
}

fn bind_console() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init().expect("Should be able to bind to console.log");
}

fn set_title(title: &str) {
    let the_window = window().expect("Expected window");
    let document: Document = the_window.document().expect("Expected document");
    document.set_title(title);
}

fn create_canvas() -> HtmlCanvasElement {
    let the_window = window().expect("Expected window");
    let document: Document = the_window.document().expect("Expected document");
    let body: HtmlElement = document.body().expect("The document should have a body");
    let canvas_element: Element = document.create_element("canvas").expect("Expected can create canvas");
    let canvas: HtmlCanvasElement = canvas_element.dyn_into::<HtmlCanvasElement>().expect("Expected canvas to be a canvas");

    canvas.set_id("knukki-canvas");

    set_canvas_size(&canvas);

    body.append_child(&canvas).expect("Should be able to insert the knukki canvas");

    canvas
}

struct ContextJSON {}

impl Serialize for ContextJSON {

    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut map = serializer.serialize_map(Some(1))?;

        // knukki relies on incremental drawing
        map.serialize_entry("preserveDrawingBuffer", &true)?;
        map.end()
    }
}

fn start_render_loop(
    canvas: &HtmlCanvasElement,
    wrap_app: Rc<RefCell<Application>>,
    force_next_render: Rc<Cell<bool>>
) {

    let the_window = window().expect("There should be a window");

    let context_options = JsValue::from_serde(&ContextJSON{})
        .expect("Should be able to create context options");

    // Create a WebGl 1 context. I'm not afraid to lose browser support if I would use WebGl 2,
    // but I am not planning to use any WebGl 2 features, so we could as well stick to WebGl 1.
    let gl_context: WebGlRenderingContext = canvas.get_context_with_context_options("webgl", &context_options)
        // The result of get_context is Result<Option<...>, ...>, so we need to expect twice
        .expect("Should be able to create WebGl context")
        .expect("Should be able to create WebGl context")
        .dyn_into::<WebGlRenderingContext>()
        .expect("Should get a WebGlRenderingContext when creating a 'webgl' canvas context");

    let glow_context = glow::Context::from_webgl1_context(gl_context);
    let golem_context = golem::Context::from_glow(glow_context)
        .expect("Should be able to turn Glow context into Golem context");

    let mut renderer = Renderer::new(
        golem_context,
        // The viewport will be set right before rendering, so this value will never be used
        RenderRegion::with_size(0, 0, 100, 100)
    );

    let mut render_function = move || {
        let scale_factor = get_scale_factor();
        let unscaled_width = get_window_width();
        let unscaled_height = get_window_height();

        let region = RenderRegion::with_size(
            0, 0,
            get_scaled(unscaled_width, scale_factor),
            get_scaled(unscaled_height, scale_factor)
        );
        renderer.reset_viewport(region);

        let mut app = wrap_app.borrow_mut();
        app.render(&renderer, force_next_render.get());

        force_next_render.set(false);
    };

    let closure_rr: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let closure_rr_inner = Rc::clone(&closure_rr);

    let render_closure = Closure::wrap(Box::new(move || {
        render_function();

        let inner_render_closure = closure_rr_inner.borrow();
        the_window.request_animation_frame(
            inner_render_closure.as_ref().unwrap().as_ref().unchecked_ref()
        ).expect("Should be able to continue requestAnimationFrame");
    }) as Box<dyn FnMut()>);

    closure_rr.replace(Some(render_closure));

    let render_closure = closure_rr.borrow();

    let the_window = window().expect("There should be a window");
    the_window.request_animation_frame(
        render_closure.as_ref().unwrap().as_ref().unchecked_ref()
    ).expect("Should be able to initiate requestAnimationFrame");
}

fn propagate_mouse_events(
    wrap_app: &Rc<RefCell<Application>>
) {
    let the_window = window().expect("Expected a window");

    fn get_x(event: &MouseEvent) -> f32 {
        let int_x = event.client_x() as f32;
        int_x / get_window_width() as f32
    }

    fn get_y(event: &MouseEvent) -> f32 {
        let int_y = event.client_y() as f32;
        1.0 - int_y / get_window_height() as f32
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
    let mouse_point_rc = Rc::new(Cell::new(None));
    let last_press_point_rc = Rc::new(Cell::new(None));

    let click_wrap_app = Rc::clone(wrap_app);
    let press_wrap_app = Rc::clone(wrap_app);
    let release_wrap_app = Rc::clone(wrap_app);
    let move_wrap_app = Rc::clone(wrap_app);
    let enter_wrap_app = Rc::clone(wrap_app);
    let leave_wrap_app = Rc::clone(wrap_app);

    let press_point_rc_press = Rc::clone(&last_press_point_rc);
    let press_point_rc_click = Rc::clone(&last_press_point_rc);
    let mouse_point_rc_move = Rc::clone(&mouse_point_rc);
    let mouse_point_rc_enter = Rc::clone(&mouse_point_rc);
    let mouse_point_rc_leave = Rc::clone(&mouse_point_rc);

    let click_closure = Closure::wrap(Box::new(move |event| {
        if let Some(press_point) = press_point_rc_click.get() {
            let click_point = Point::new(get_x(&event), get_y(&event));

            // I don't want to count drags as clicks, so I only fire the event if the point of
            // clicking/release is close enough to the point where the mouse was pressed.
            if click_point.distance_to(press_point) < 0.1 {
                let mut app = click_wrap_app.borrow_mut();
                app.fire_mouse_click_event(MouseClickEvent::new(
                    primary_mouse,
                    click_point,
                    get_button(&event)
                ));
            }
        }
    }) as Box<dyn FnMut(MouseEvent)>);

    let press_closure = Closure::wrap(Box::new(move |event| {
        let mut app = press_wrap_app.borrow_mut();
        let point = Point::new(get_x(&event), get_y(&event));
        app.fire_mouse_press_event(MousePressEvent::new(
            primary_mouse,
            point,
            get_button(&event)
        ));
        press_point_rc_press.set(Some(point));
    }) as Box<dyn FnMut(MouseEvent)>);

    let release_closure = Closure::wrap(Box::new(move |event| {
        let mut app = release_wrap_app.borrow_mut();
        app.fire_mouse_release_event(MouseReleaseEvent::new(
            primary_mouse,
            Point::new(get_x(&event), get_y(&event)),
            get_button(&event)
        ));
    }) as Box<dyn FnMut(MouseEvent)>);

    let move_closure = Closure::wrap(Box::new(move |event| {
        let old_mouse_point = mouse_point_rc_move.get();
        let new_mouse_point = Point::new(get_x(&event), get_y(&event));

        if let Some(prev_mouse_point) = old_mouse_point {

            // Protect the Application from 0-length move events
            if prev_mouse_point != new_mouse_point {
                let mut app = move_wrap_app.borrow_mut();
                app.fire_mouse_move_event(MouseMoveEvent::new(
                    primary_mouse, prev_mouse_point, new_mouse_point
                ));
            }
        }

        mouse_point_rc_move.set(Some(new_mouse_point));
    }) as Box<dyn FnMut(MouseEvent)>);

    let enter_closure = Closure::wrap(Box::new(move |event| {

        // If we somehow lost a leave event, we should pretend it never happened
        // This is to prevent the Application from unexpected event flows
        if mouse_point_rc_enter.get().is_none() {
            let entrance_mouse_point = Point::new(get_x(&event), get_y(&event));

            let mut app = enter_wrap_app.borrow_mut();
            app.fire_mouse_enter_event(MouseEnterEvent::new(
                primary_mouse, entrance_mouse_point
            ));

            mouse_point_rc_enter.set(Some(entrance_mouse_point));
        }
    }) as Box<dyn FnMut(MouseEvent)>);

    let leave_closure = Closure::wrap(Box::new(move |event| {
        let old_mouse_pos = mouse_point_rc_leave.get();
        let exit_point = Point::new(get_x(&event), get_y(&event));

        // It would be weird if there were no old mouse pos, but let's not panic for that
        if let Some(old_mouse_pos) = old_mouse_pos {
            let mut app = leave_wrap_app.borrow_mut();

            // Mouse leave events sometimes occur outside the browser window. We shouldn't fire
            // move events to such places to the Application
            if exit_point.get_x() >= 0.0 && exit_point.get_x() <= 1.0
                && exit_point.get_y() >= 0.0 && exit_point.get_y() <= 1.0 {

                if exit_point != old_mouse_pos {
                    app.fire_mouse_move_event(MouseMoveEvent::new(
                        primary_mouse, old_mouse_pos, exit_point
                    ));
                }

                app.fire_mouse_leave_event(MouseLeaveEvent::new(
                    primary_mouse, exit_point
                ));
            } else {

                // Let's use the last valid mouse position as back-up exit point
                app.fire_mouse_leave_event(MouseLeaveEvent::new(
                    primary_mouse, old_mouse_pos
                ));
            }
        }

        mouse_point_rc_leave.set(None);
    }) as Box<dyn FnMut(MouseEvent)>);

    let context_closure = Closure::wrap(Box::new(|event: Event| {
        event.prevent_default();
    }) as Box<dyn FnMut(Event)>);

    the_window.add_event_listener_with_callback("click", click_closure.as_ref().unchecked_ref())
        .expect("Should be able to add click listener");
    the_window.add_event_listener_with_callback("auxclick", click_closure.as_ref().unchecked_ref())
        .expect("Should be able to add auxclick listener");
    the_window.add_event_listener_with_callback("mousedown", press_closure.as_ref().unchecked_ref())
        .expect("Should be able to add mousedown listener");
    the_window.add_event_listener_with_callback("mouseup", release_closure.as_ref().unchecked_ref())
        .expect("Should be able to add mouseup listener");
    the_window.add_event_listener_with_callback("mousemove", move_closure.as_ref().unchecked_ref())
        .expect("Should be able to add mousemove listener");
    the_window.add_event_listener_with_callback("mouseover", enter_closure.as_ref().unchecked_ref())
        .expect("Should be able to add mouseover listener");
    the_window.add_event_listener_with_callback("mouseout", leave_closure.as_ref().unchecked_ref())
        .expect("Should be able to add mouseout listener");
    the_window.add_event_listener_with_callback("contextmenu", context_closure.as_ref().unchecked_ref())
        .expect("Should be able to add contextmenu listener");

    click_closure.forget();
    press_closure.forget();
    release_closure.forget();
    move_closure.forget();
    enter_closure.forget();
    leave_closure.forget();
    context_closure.forget();
}

fn maintain_canvas_size(canvas: &HtmlCanvasElement, force_next_render: Rc<Cell<bool>>) {
    let the_window = window().expect("Expected a window");

    // Note: This is a clone of a reference to the JS canvas; not a clone of the actual canvas
    let canvas = canvas.clone();

    let resize_closure = Closure::wrap(Box::new(move || {
        set_canvas_size(&canvas);
        force_next_render.set(true);
        // TODO Fire resize event
    }) as Box<dyn FnMut()>);

    the_window.add_event_listener_with_callback(
        "resize", resize_closure.as_ref().unchecked_ref()
    ).expect("Should be able to add resize listener");

    resize_closure.forget();
}

fn set_canvas_size(canvas: &HtmlCanvasElement) {
    let unscaled_width = get_window_width();
    let unscaled_height = get_window_height();
    let scale_factor = get_scale_factor();

    canvas.set_width(get_scaled(unscaled_width, scale_factor));
    canvas.set_height(get_scaled(unscaled_height, scale_factor));

    let canvas_style = canvas.style();
    canvas_style.set_property("width", &format!("{}px", unscaled_width))
        .expect("Should be able to set canvas CSS width");
    canvas_style.set_property("height", &format!("{}px", unscaled_height))
        .expect("Should be able to set canvas CSS height");
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

fn get_scale_factor() -> f64 {
    let the_window = window().expect("There should be a window");
    the_window.device_pixel_ratio()
}

fn get_scaled(size: u32, scale_factor: f64) -> u32 {
    ((size as f64 * scale_factor) as u32).max(10)
}