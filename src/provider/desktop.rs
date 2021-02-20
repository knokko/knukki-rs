use crate::{Application, MouseEnterEvent, MouseLeaveEvent, MouseMoveEvent, RenderRegion, Renderer, MousePressEvent};

use golem::*;

use glutin::{
    dpi::PhysicalPosition,
    dpi::PhysicalSize,
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
    window::WindowBuilder,
    ContextWrapper, PossiblyCurrent,
};

use golem::Dimension::D2;
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

    let mut renderer = Renderer::new(
        // The initial viewport doesn't matter in this situation because it will be overwritten
        // before rendering anyway
        golem,
        RenderRegion::with_size(0, 0, 1, 1),
    );

    let mut copy_pack =
        create_copy_pack(renderer.get_context()).expect("Should be able to create copy pack");

    let mut start_time = Instant::now();

    let mut mouse_position: Option<PhysicalPosition<i32>> = None;
    let mut pressed_buttons = Vec::with_capacity(2);
    let mut should_fire_mouse_enter_event = false;

    let mut render_surface: Option<Surface> = None;

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
                        render_surface = None;
                    }
                    WindowEvent::MouseInput {
                        device_id: _,
                        state,
                        button,
                        ..
                    } => {
                        if state == ElementState::Released || state == ElementState::Pressed {

                            // Convert winit button to knukki button
                            let knukki_button = match button {
                                MouseButton::Left => crate::MouseButton::primary(),
                                MouseButton::Right => crate::MouseButton::new(1),
                                MouseButton::Middle => crate::MouseButton::new(2),
                                MouseButton::Other(id) => crate::MouseButton::new(id),
                            };

                            if state == ElementState::Pressed {
                                pressed_buttons.push(knukki_button);
                            } else {
                                pressed_buttons.retain(|pressed_button| *pressed_button != knukki_button);
                            }

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

                                // Construct and fire the events
                                if state == ElementState::Pressed {
                                    let knukki_press_event = crate::MousePressEvent::new(
                                        knukki_mouse,
                                        knukki_point,
                                        knukki_button
                                    );

                                    app.fire_mouse_press_event(knukki_press_event);
                                } else {
                                    let knukki_release_event = crate::MouseReleaseEvent::new(
                                        knukki_mouse,
                                        knukki_point,
                                        knukki_button
                                    );
                                    let knukki_click_event = crate::MouseClickEvent::new(
                                        knukki_mouse,
                                        knukki_point,
                                        knukki_button,
                                    );

                                    app.fire_mouse_release_event(knukki_release_event);
                                    app.fire_mouse_click_event(knukki_click_event);
                                }
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
                        // Also, winit seems to fire mouse move events outside the window if a mouse
                        // button is pressed.
                        let window_size = windowed_context.window().inner_size();
                        if position.x <= 0
                            || position.y <= 0
                            || (position.x as u32) >= window_size.width
                            || (position.y as u32) >= window_size.height
                        {
                            return;
                        }

                        if should_fire_mouse_enter_event {
                            let x = position.x as f32 / window_size.width as f32;
                            let y = 1.0 - position.y as f32 / window_size.height as f32;
                            let mouse = crate::Mouse::new(0);
                            let entrance_point = crate::Point::new(x, y);

                            let event = MouseEnterEvent::new(
                                mouse,
                                entrance_point
                            );
                            app.fire_mouse_enter_event(event);
                            should_fire_mouse_enter_event = false;

                            // Also fire press events for all buttons that are pressed
                            for button in &pressed_buttons {
                                app.fire_mouse_press_event(MousePressEvent::new(
                                    mouse, entrance_point, *button
                                ));
                            }
                        }

                        // If there is a previous mouse position, fire a move event
                        if let Some(previous_position) = mouse_position {
                            // Winit seems to fire a double cursor move event when the cursor enters
                            // the window. I don't know if this happens more often, but let's be
                            // careful and not propagate move events between equal positions.
                            if previous_position.x != position.x
                                || previous_position.y != position.y
                            {
                                let old_x = previous_position.x as f32 / window_size.width as f32;
                                let old_y =
                                    1.0 - previous_position.y as f32 / window_size.height as f32;
                                let new_x = position.x as f32 / window_size.width as f32;
                                let new_y = 1.0 - position.y as f32 / window_size.height as f32;
                                let event = MouseMoveEvent::new(
                                    crate::Mouse::new(0),
                                    crate::Point::new(old_x, old_y),
                                    crate::Point::new(new_x, new_y),
                                );
                                app.fire_mouse_move_event(event);
                            }
                        }

                        mouse_position = Some(position);
                    }
                    WindowEvent::CursorEntered { .. } => {
                        should_fire_mouse_enter_event = true;
                    }
                    WindowEvent::CursorLeft { .. } => {
                        // If we know where the cursor was, we should fire a MouseLeaveEvent
                        if let Some(previous_position) = mouse_position {
                            let window_size = windowed_context.window().inner_size();
                            let old_x = previous_position.x as f32 / window_size.width as f32;
                            let old_y =
                                1.0 - previous_position.y as f32 / window_size.height as f32;
                            let event = MouseLeaveEvent::new(
                                crate::Mouse::new(0),
                                crate::Point::new(old_x, old_y),
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

                // Give the application a render opportunity every ~16 milliseconds
                let current_time = Instant::now();
                let elapsed_time = (current_time - start_time).as_millis();
                if elapsed_time < 16 {
                    sleep(Duration::from_millis(16 - elapsed_time as u64));
                }
                start_time = Instant::now();

                draw_application(
                    &mut app,
                    &mut renderer,
                    &mut copy_pack,
                    &mut render_surface,
                    size,
                    force,
                    &windowed_context,
                )
                .expect("Should be able to draw app");
            }
            Event::RedrawRequested(_) => {
                // This provider will never request a winit redraw, so when this
                // event is fired, it must have come from the OS.
                let force = true;

                // Draw onto the entire inner window buffer
                let size = windowed_context.window().inner_size();

                draw_application(
                    &mut app,
                    &mut renderer,
                    &mut copy_pack,
                    &mut render_surface,
                    size,
                    force,
                    &windowed_context,
                )
                .expect("Should be able to force draw app");
            }
            _ => (),
        }
    });

    fn draw_application(
        app: &mut Application,
        renderer: &mut Renderer,
        copy_pack: &mut (ShaderProgram, VertexBuffer, ElementBuffer),
        render_surface: &mut Option<Surface>,
        size: PhysicalSize<u32>,
        force: bool,
        windowed_context: &ContextWrapper<PossiblyCurrent, Window>,
    ) -> Result<(), GolemError> {
        let region = RenderRegion::with_size(0, 0, size.width, size.height);

        let mut created_surface = false;

        // Make sure there is an up-to-date render texture to draw the application on
        if render_surface.is_none() {
            let mut render_texture =
                Texture::new(renderer.get_context()).expect("Should be able to create texture");
            render_texture.set_image(None, size.width, size.height, ColorFormat::RGBA);
            *render_surface = Some(
                Surface::new(renderer.get_context(), render_texture)
                    .expect("Should be able to create surface"),
            );
            created_surface = true;
            render_surface.as_ref().unwrap().bind();
        }

        // Draw the application on the render texture
        let render_surface = render_surface.as_ref().unwrap();
        renderer.reset_viewport(region);
        if app.render(&renderer, force || created_surface) {
            // Draw the render texture onto the presenting texture
            Surface::unbind(renderer.get_context());
            renderer
                .get_context()
                .set_viewport(0, 0, size.width, size.height);
            renderer.get_context().disable_scissor();

            let shader = &mut copy_pack.0;
            let vb = &mut copy_pack.1;
            let eb = &mut copy_pack.2;

            shader.bind();
            shader.prepare_draw(&vb, &eb)?;

            let bind_point = std::num::NonZeroU32::new(1).unwrap();
            unsafe {
                let texture = render_surface.borrow_texture().unwrap();
                texture.set_active(bind_point);
            }
            unsafe {
                // There are always 6 indices when there are 2 triangles, like in this case
                shader.draw_prepared(0..6, GeometryMode::Triangles);
            }

            windowed_context.swap_buffers().expect("Good context");

            render_surface.bind();
        }
        Ok(())
    }

    fn create_copy_pack(
        golem: &Context,
    ) -> Result<(ShaderProgram, VertexBuffer, ElementBuffer), GolemError> {
        let mut vb = VertexBuffer::new(&golem)?;
        let mut eb = ElementBuffer::new(&golem)?;

        #[rustfmt::skip]
            let vertices = [
            -1.0, -1.0,
            1.0, -1.0,
            1.0, 1.0,
            -1.0, 1.0,
        ];
        let indices = [0, 1, 2, 2, 3, 0];
        let mut shader = ShaderProgram::new(
            &golem,
            ShaderDescription {
                vertex_input: &[Attribute::new("position", AttributeType::Vector(D2))],
                fragment_input: &[Attribute::new("passPosition", AttributeType::Vector(D2))],
                uniforms: &[Uniform::new("image", UniformType::Sampler2D)],
                vertex_shader: r#" void main() {
            gl_Position = vec4(position.x, position.y, 0.0, 1.0);
            passPosition = position;
        }"#,
                fragment_shader: r#" void main() {
            vec4 theColor = texture(image, vec2(0.5 + passPosition.x * 0.5, 0.5 + passPosition.y * 0.5));
            gl_FragColor = theColor;
        }"#,
            },
        )?;
        vb.set_data(&vertices);
        eb.set_data(&indices);
        shader.bind();
        shader.set_uniform("image", UniformValue::Int(1))?;

        Ok((shader, vb, eb))
    }
}
