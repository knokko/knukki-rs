use crate::*;

use golem::Context;

use glutin::{
    event::{Event, WindowEvent},
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

    println!("Start the event loop...");
    event_loop.run(move |event, _target, control_flow| {
    
        // TODO Look into the details of this behavior!
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                println!("The close button was pressed; stopping");
                *control_flow = ControlFlow::Exit
            },
            Event::MainEventsCleared => {
                // Let the application decide whether it needs to redraw itself
                //println!("Main events cleared");
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
                    windowed_context.swap_buffers();
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
                windowed_context.swap_buffers();
            },
            _ => ()
        }
    });
}