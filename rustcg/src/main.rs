#[macro_use]
extern crate gfx;
extern crate glutin;
extern crate gfx_core;
extern crate gfx_window_glutin;

use glutin::dpi::*;
use glutin::*;

use gfx_core::format::{DepthStencil, Rgba8};
use std::path::Path;
use gfx_core::Device;
use gfx::traits::FactoryExt;

// Define pipeline `pipe`
gfx_defines! {
    vertex Vertex {
        pos: [f32; 3] = "aPos",
    }
    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        out_color: gfx::RenderTarget<gfx::format::Rgba8> = "Target0",
    }
}

// Add a utility ctor for Vertex
impl Vertex {
    fn new(p: [f32; 3]) -> Vertex {
        Vertex {
            pos: p
        }
    }
}

fn load_shader(name: &str) -> Vec<u8> {
    let path = Path::new("./assets/").join(name);
    return std::fs::read(path).unwrap();
}

fn main() {
    // Init glutin
    let mut events_loop = glutin::EventsLoop::new();
    let window_builder = glutin::WindowBuilder::new()
        .with_title("WeAthFolD's gfx playground")
        .with_dimensions(LogicalSize::new(1280.0, 720.0));

    // GL context
    let context = glutin::ContextBuilder::new();

    // Setup gfx_window_glutin
    let (window, mut device, mut factory, mut rtv, mut stv) =
        gfx_window_glutin::init::<Rgba8, DepthStencil>(window_builder, context, &events_loop).unwrap();

    // Create Encoder (i.e. command buffer)
    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();
    // Create pso
    let pso = factory.create_pipeline_simple(
        load_shader("00-triangle.vert").as_slice(),
        load_shader("00-triangle.frag").as_slice(),
        pipe::new()
    ).unwrap();

    // Vertex data
    let triangle: [Vertex; 3] = [
        Vertex::new([0.5, -0.5, 0.0]),
        Vertex::new([-0.5, -0.5, 0.0]),
        Vertex::new([-0.5, 0.5, 0.0]),
    ];

    // Create VBO
    let (vertex_buffer, slice) = factory.create_vertex_buffer_with_slice(&triangle, ());

    // Make window current, start main loop
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
                        // Here resized new views will be created
                        gfx_window_glutin::update_views(&window, &mut rtv, &mut stv);
                    },
                    _ => ()
                },
                _ => ()
            }
        });

        // Emit draw calls
        // !! Note that vertex_buffer and rtv are all HANDLES to underlying buffer,
        //   and here we DUPLICATE the handle.
        let pipe_data = pipe::Data {
            vbuf: vertex_buffer.clone(),
            out_color: rtv.clone(),
        };
        encoder.clear(&rtv, [0.2, 0.2, 0.3, 1.0]);
        encoder.draw(&slice, &pso, &pipe_data);

        // Flush
        encoder.flush(&mut device);

        // Swap buffers
        window.swap_buffers().unwrap();
        device.cleanup();
    }
}
