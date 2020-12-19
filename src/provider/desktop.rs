use crate::{
    Application,
    RenderRegion
};

use golem::Context;

use glutin::{
    event::{Event, WindowEvent, ElementState, MouseButton},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    dpi::PhysicalPosition
};

use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;

pub fn start(mut app: Application, title: &str) {
    let event_loop = EventLoop::new();
    let builder = WindowBuilder::new()
        .with_decorations(true)
        .with_maximized(false)
        .with_resizable(true)
        .with_title(title)
        .with_visible(true)
    ;
    let windowed_context = unsafe { glutin::ContextBuilder::new()
        .build_windowed(builder, &event_loop)
        .expect("Should be able to create a window")
        .make_current().expect("Should be able to make context current")
    };

    let golem = Context::from_glow(glow::Context::from_loader_function(
        |function_name| windowed_context.get_proc_address(function_name)
    )).expect("Should be able to create Golem context");

    let mut start_time = Instant::now();

    let mut mouse_position: Option<PhysicalPosition<i32>> = None;

    event_loop.run(move |event, _target, control_flow| {
    
        // I use `Poll` instead of `Wait` to get more control over the control flow.
        // I use a simple custom system to avoid too large power usage
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit
            },
            Event::WindowEvent{window_id: _, event: window_event} => {
                match window_event {
                    WindowEvent::Resized(_) => {
                        // TODO app.on_resize
                    },
                    WindowEvent::MouseInput{device_id: _, state, button, ..} => {
                        if state == ElementState::Released {

                            // It would be weird if we don't have a mouse position
                            if let Some(click_position) = mouse_position {

                                // Just 1 mouse on desktops
                                let knukki_mouse = crate::Mouse::new(0);

                                // Convert winit mouse position to knukki mouse position
                                let window_size = windowed_context.window().inner_size();
                                let knukki_x = click_position.x as f32 / window_size.width as f32;
                                let knukki_y = 1.0 - (click_position.y as f32 / window_size.height as f32);
                                let knukki_point = crate::MousePoint::new(knukki_x, knukki_y);

                                // Convert winit button to knukki button
                                let knukki_button = match button {
                                    MouseButton::Left => crate::MouseButton::primary(),
                                    MouseButton::Right => crate::MouseButton::new(1),
                                    MouseButton::Middle => crate::MouseButton::new(2),
                                    MouseButton::Other(id) => crate::MouseButton::new(id)
                                };

                                // Construct and fire the event
                                let knukki_event = crate::MouseClickEvent::new(
                                    knukki_mouse, knukki_point, knukki_button
                                );

                                app.fire_mouse_click_event(knukki_event);
                            }
                        }
                    },
                    WindowEvent::CursorMoved{device_id: _, position, ..} => {
                        // We need to remember the mouse position for click events
                        mouse_position = Some(position);
                        // TODO Fire mouse_move_event
                    },
                    _ => ()
                }
            },
            Event::MainEventsCleared => {
                // Let the application decide whether it needs to redraw itself
                let force = false;

                // Draw onto the entire inner window buffer
                let size = windowed_context.window().inner_size();
                let region = RenderRegion::with_size(0, 0, size.width, size.height);

                // Give the application a render opportunity every ~16 milliseconds
                let current_time = Instant::now();
                let elapsed_time = (current_time - start_time).as_millis();
                if elapsed_time < 16 {
                    sleep(Duration::from_millis(16 - elapsed_time as u64));
                }
                start_time = Instant::now();

                // Only swap the buffers if the application actually rendered
                if app.render(&golem, region, force) {
                    windowed_context.swap_buffers().expect("Good context");
                }

                
            },
            Event::RedrawRequested(_) => {
                // This provider will never request a winit redraw, so when this
                // event is fired, it must have come from the OS.
                let force = true;

                // Draw onto the entire inner window buffer
                let size = windowed_context.window().inner_size();
                let region = RenderRegion::with_size(0, 0, size.width, size.height);

                app.render(&golem, region, force);
                windowed_context.swap_buffers().expect("Good context");
            },
            _ => ()
        }
    });
}