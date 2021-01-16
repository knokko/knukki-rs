use crate::{Application, RenderRegion, MouseMoveEvent, MouseLeaveEvent, MouseEnterEvent, GolemRenderer};

use golem::Context;

use glutin::{
    dpi::PhysicalPosition,
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
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
        .with_visible(true);
    let windowed_context = unsafe {
        glutin::ContextBuilder::new()
            .build_windowed(builder, &event_loop)
            .expect("Should be able to create a window")
            .make_current()
            .expect("Should be able to make context current")
    };

    let golem = Context::from_glow(glow::Context::from_loader_function(|function_name| {
        windowed_context.get_proc_address(function_name)
    }))
    .expect("Should be able to create Golem context");
    let renderer = GolemRenderer::new(golem);

    let mut start_time = Instant::now();

    let mut mouse_position: Option<PhysicalPosition<i32>> = None;
    let mut should_fire_mouse_enter_event = false;

    event_loop.run(move |event, _target, control_flow| {
        // I use `Poll` instead of `Wait` to get more control over the control flow.
        // I use a simple custom system to avoid too large power usage
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                window_id: _,
                event: window_event,
            } => {
                match window_event {
                    WindowEvent::Resized(_) => {
                        // TODO app.on_resize
                    }
                    WindowEvent::MouseInput {
                        device_id: _,
                        state,
                        button,
                        ..
                    } => {
                        if state == ElementState::Released {
                            // It would be weird if we don't have a mouse position
                            if let Some(click_position) = mouse_position {
                                // Just 1 mouse on desktops
                                let knukki_mouse = crate::Mouse::new(0);

                                // Convert winit mouse position to knukki mouse position
                                let window_size = windowed_context.window().inner_size();
                                let knukki_x = click_position.x as f32 / window_size.width as f32;
                                let knukki_y =
                                    1.0 - (click_position.y as f32 / window_size.height as f32);
                                let knukki_point = crate::Point::new(knukki_x, knukki_y);

                                // Convert winit button to knukki button
                                let knukki_button = match button {
                                    MouseButton::Left => crate::MouseButton::primary(),
                                    MouseButton::Right => crate::MouseButton::new(1),
                                    MouseButton::Middle => crate::MouseButton::new(2),
                                    MouseButton::Other(id) => crate::MouseButton::new(id),
                                };

                                // Construct and fire the event
                                let knukki_event = crate::MouseClickEvent::new(
                                    knukki_mouse,
                                    knukki_point,
                                    knukki_button,
                                );

                                app.fire_mouse_click_event(knukki_event);
                            }
                        }
                    }
                    WindowEvent::CursorMoved {
                        device_id: _,
                        position,
                        ..
                    } => {
                        // Winit seems to fire mouse move events in occasions like clicking on the
                        // app icon in the taskbar or opening the window, even when the cursor is
                        // not inside the window. Let's just ignore these events.
                        let window_size = windowed_context.window().inner_size();
                        if position.x < 0 || position.y < 0 ||
                            (position.x as u32) >= window_size.width ||
                            (position.y as u32) >= window_size.height {
                            return;
                        }
                        // If there is a previous mouse position, fire a move event
                        if let Some(previous_position) = mouse_position {
                            // Winit seems to fire a double cursor move event when the cursor enters
                            // the window. I don't know if this happens more often, but let's be
                            // careful and not propagate move events between equal positions.
                            if previous_position.x != position.x || previous_position.y != position.y {
                                let old_x = previous_position.x as f32 / window_size.width as f32;
                                let old_y = 1.0 - previous_position.y as f32 / window_size.height as f32;
                                let new_x = position.x as f32 / window_size.width as f32;
                                let new_y = 1.0 - position.y as f32 / window_size.height as f32;
                                let event = MouseMoveEvent::new(
                                    crate::Mouse::new(0),
                                    crate::Point::new(old_x, old_y),
                                    crate::Point::new(new_x, new_y)
                                );
                                app.fire_mouse_move_event(event);
                            }
                        } else {
                            if should_fire_mouse_enter_event {
                                let x = position.x as f32 / window_size.width as f32;
                                let y = 1.0 - position.y as f32 / window_size.height as f32;
                                let event = MouseEnterEvent::new(
                                    crate::Mouse::new(0), crate::Point::new(x, y)
                                );
                                app.fire_mouse_enter_event(event);
                                should_fire_mouse_enter_event = false;
                            }
                        }
                        mouse_position = Some(position);
                    }
                    WindowEvent::CursorEntered { .. } => {
                        should_fire_mouse_enter_event = true;
                    },
                    WindowEvent::CursorLeft { .. } => {
                        // If we know where the cursor was, we should fire a MouseLeaveEvent
                        if let Some(previous_position) = mouse_position {
                            let window_size = windowed_context.window().inner_size();
                            let old_x = previous_position.x as f32 / window_size.width as f32;
                            let old_y = 1.0 - previous_position.y as f32 / window_size.height as f32;
                            let event = MouseLeaveEvent::new(
                                crate::Mouse::new(0), crate::Point::new(old_x, old_y)
                            );
                            app.fire_mouse_leave_event(event);
                        }

                        // Once the mouse leaves the window, we have no clue where it is, but it
                        // won't be at this mouse position
                        mouse_position = None;
                    }
                    _ => (),
                }
            }
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
                if app.render(&renderer, region, force) {
                    windowed_context.swap_buffers().expect("Good context");
                }
            }
            Event::RedrawRequested(_) => {
                // This provider will never request a winit redraw, so when this
                // event is fired, it must have come from the OS.
                let force = true;

                // Draw onto the entire inner window buffer
                let size = windowed_context.window().inner_size();
                let region = RenderRegion::with_size(0, 0, size.width, size.height);

                app.render(&renderer, region, force);
                windowed_context.swap_buffers().expect("Good context");
            }
            _ => (),
        }
    });
}
