#[macro_use]
extern crate gfx;
extern crate rustcg;
extern crate gfx_core;

use rustcg::*;
use gfx::traits::FactoryExt;
use gfx_core::factory::Factory;

gfx_defines! {
    vertex Vertex {
        pos: [f32; 2] = "aPos",
    }
    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        out_color: gfx::RenderTarget<gfx::format::Rgba8> = "Target0",
        tex: gfx::TextureSampler<[f32; 4]> = "tex",
    }
}

impl Vertex {
    fn new(p: [f32; 2]) -> Vertex { Vertex { pos: p } }
}


#[derive(Copy, Clone, Debug)]
struct Color {
    r: u8, g: u8, b: u8, a: u8
}

impl Into<u32> for Color {
    fn into(self) -> u32 {
        (self.r as u32) << 24 | (self.g as u32) << 16 | (self.b as u32) << 8 | (self.a as u32)
    }
}

impl Into<Color> for u32 {
    fn into(self) -> Color {
        Color::new(
            (self >> 24 & 0xff) as u8,
            (self >> 16 & 0xff) as u8,
            (self >> 8  & 0xff) as u8,
            (self       & 0xff) as u8)
    }
}

impl Color {
    fn new(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color {
            r, g, b, a
        }
    }
}

struct Image {
    width: u32,
    height: u32,
    data: Vec<u8>
}

impl Image {
    fn new(w: u32, h: u32) -> Image {
        let mut vec = Vec::new();
        for _ in 0 .. w * h * 4 {
            vec.push(0);
        }

        Image {
            width: w,
            height: h,
            data: vec
        }
    }

    fn put_pixel(&mut self, x: u32, y: u32, c: Color) {
        let off = 4 * (y * self.width + x) as usize;
        let ref mut d = &mut self.data;
        d[off]     = c.r;
        d[off + 1] = c.g;
        d[off + 2] = c.b;
        d[off + 3] = c.a;
    }

    fn get_pixel(&self, x: u32, y: u32) -> Color {
        let off = 4 * (y * self.width + x) as usize;
        let ref d = self.data;
        Color::new(d[off], d[off + 1], d[off + 2], d[off + 3])
    }
}

fn apply_raytrace(img: &mut Image) {
    for x in 0 .. img.width {
        for y in 0 .. img.height {
            let (w, h) = (img.width, img.height);
            img.put_pixel(x, y, Color::new((x * 255 / w) as u8, (y * 255 / h) as u8, 128, 255));
        }
    }
}

fn main() {
    let mut ctx = GameContext::init();

    let (mut encoder, pso, vbo, slice) = {
        let mut gfx = &mut ctx.gfx;

        // Create Encoder (i.e. command buffer)
        let mut encoder: gfx::Encoder<_, _> = gfx.factory.create_command_buffer().into();

        // Create pso
        let pso = gfx.factory.create_pipeline_simple(
            rustcg::resource::load_bytes("02-raytracer.vert").as_slice(),
            rustcg::resource::load_bytes("02-raytracer.frag").as_slice(),
            pipe::new()
        ).unwrap();

        // Vertex data
        let triangle: [Vertex; 4] = [
            Vertex::new([-1.0, -1.0]),
            Vertex::new([-1.0, 1.0]),
            Vertex::new([1.0, 1.0]),
            Vertex::new([1.0, -1.0]),
        ];

        // Create VBO
        let indices = vec![0u32, 1, 2, 0, 2, 3];
        let (vertex_buffer, slice) = gfx.factory.create_vertex_buffer_with_slice(&triangle, &*indices);
        (encoder, pso, vertex_buffer, slice)
    };

    let mut running = true;
    let (w, h) = ctx.get_window_size();
    let (w, h) = (w as u16, h as u16);
    let mut image = Image::new(w as u32, h as u32);
    while running {
        ctx.process_events(&mut running);
        {
            let ref mut g = ctx.gfx;

            apply_raytrace(&mut image);
            let texture = {
                let kind = gfx::texture::Kind::D2(w, h, gfx::texture::AaMode::Single);
                let (_, view) = g.factory.create_texture_immutable_u8::<gfx::format::Rgba8>(
                    kind, gfx::texture::Mipmap::Allocated, &[&*image.data]).unwrap();
                view
            };

            let sampler = g.factory.create_sampler_linear();

            let pipe_data = pipe::Data {
                vbuf: vbo.clone(),
                out_color: g.color_view.clone(),
                tex: (texture, sampler)
            };

            encoder.draw(&slice, &pso, &pipe_data);
            encoder.flush(&mut g.device);
        }
        ctx.frame_end();
    }
}

