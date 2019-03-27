#[macro_use]
extern crate gfx;
extern crate rustcg;
extern crate cgmath;

use cgmath::Transform;
use rustcg::*;
use gfx::traits::FactoryExt;
use gfx::IntoIndexBuffer;

// Define pipeline `pipe`
gfx_defines! {
    vertex Vertex {
        pos: [f32; 3] = "aPos",
    }

    constant Locals {
        transform: [[f32; 4]; 4] = "uMat",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        out_color: gfx::RenderTarget<gfx::format::Rgba8> = "Target0",
        transform: gfx::ConstantBuffer<Locals> = "Transform",
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

fn main() {
    let mut ctx = GameContext::init();

    let (mut encoder, pso, vertex_buffer, slice) = {
        let mut gfx = &mut ctx.gfx;

        // Create Encoder (i.e. command buffer)
        let mut encoder: gfx::Encoder<_, _> = gfx.factory.create_command_buffer().into();

        // Create pso
        let pso = gfx.factory.create_pipeline_simple(
            rustcg::resource::load_bytes("01-projection.vert").as_slice(),
            rustcg::resource::load_bytes("00-triangle.frag").as_slice(),
            pipe::new()
        ).unwrap();

        // Vertex data
        let triangle: [Vertex; 3] = [
            Vertex::new([0.5, -0.5, -1.0]),
            Vertex::new([-0.5, -0.5, -1.0]),
            Vertex::new([-0.5, 0.5, -1.0]),
        ];

        // Create VBO
        let indices = vec![0u32, 1, 2];
        let (vertex_buffer, slice) = gfx.factory
            .create_vertex_buffer_with_slice(&triangle, &*indices);
        (encoder, pso, vertex_buffer, slice)
    };

    let transform_buffer = ctx.gfx.factory.create_constant_buffer(1);
    let mut running = true;
    println!("Start running");
    while running {
        ctx.process_events(&mut running);
        //// begin frame process

        let (width, height) = ctx.get_window_size();
        {
            let ref mut gfx = ctx.gfx;

            // Emit draw calls
            // !! Note that vertex_buffer and rtv are all HANDLES to underlying buffer,
            //   and here we DUPLICATE the handle.
            let pipe_data = pipe::Data {
                vbuf: vertex_buffer.clone(),
                out_color: gfx.color_view.clone(),
                transform: transform_buffer.clone()
            };
            encoder.clear(&gfx.color_view, [0.2, 0.2, 0.3, 1.0]);

            let aspect = width / height;

            let mat: [[f32; 4]; 4] = cgmath::perspective(cgmath::Deg(90.0), aspect, 0.001, 1000.0).into();

            encoder.update_buffer(&pipe_data.transform, &[Locals { transform: mat }], 0).unwrap();
            encoder.draw(&slice, &pso, &pipe_data);

            // Flush
            encoder.flush(&mut gfx.device);
        }

        //// end frame 
        ctx.frame_end();
    }
}
