extern crate glutin;
extern crate gfx_core;
extern crate gfx_window_glutin;

use glutin::dpi::*;
use glutin::*;

use gfx_core::format::{DepthStencil, Rgba8};

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let window_builder = glutin::WindowBuilder::new()
        .with_title("WeAthFolD's gfx playground")
        .with_dimensions(LogicalSize::new(1280.0, 720.0));
    let context = glutin::ContextBuilder::new();

    let (window, device, factory, rtv, stv) =
        gfx_window_glutin::init::<Rgba8, DepthStencil>(window_builder, context, &events_loop).unwrap();

    unsafe { window.make_current() }.unwrap();

    let mut running = true;
    while running {
        events_loop.poll_events(|event| {
            match event {
                glutin::Event::WindowEvent { event, .. } => match event {
                    glutin::WindowEvent::CloseRequested => running = false,
                    glutin::WindowEvent::Resized(new_size) => {
                        let dpi_factor = window.get_hidpi_factor();
                        window.resize(new_size.to_physical(dpi_factor));
                    },
                    _ => ()
                },
                _ => ()
            }
        });

        window.swap_buffers().unwrap();
    }
}
