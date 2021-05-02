#[macro_use]
extern crate gfx;
extern crate rustcg;
extern crate gfx_core;
extern crate cgmath;

use std::option::*;
use rustcg::*;
use gfx::traits::FactoryExt;
use gfx_core::factory::Factory;
use cgmath::prelude::*;
use cgmath::vec3;
use cgmath::Deg;
use cgmath::Rad;
use std::num;

type Vec3 = cgmath::Vector3<f32>;


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
    fn from_float(r: f32, g: f32, b: f32, a: f32) -> Color {
        let go = |x: f32| (x * 255.0) as u8;
        Color {
            r: go(r), g: go(g), b: go(b), a: go(a)
        }
    }

    fn new(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color {
            r, g, b, a
        }
    }

    fn lerp(c1: Color, c2: Color, t: f32) -> Color {
        let lerpi = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t) as u8;
        Color::new(
            lerpi(c1.r, c2.r),
            lerpi(c1.g, c2.g),
            lerpi(c1.b, c2.b),
            lerpi(c1.a, c2.a)
        )
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

#[derive(Copy, Clone, Debug)]
struct Ray {
    pos: Vec3,
    dir: Vec3
}

impl Ray {
    fn new(pos: Vec3, dir: Vec3) -> Ray {
        Ray { pos, dir: dir.normalize() }
    }

    fn move_by(&self, t: f32) -> Vec3 {
        self.pos + self.dir * t
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

fn ray_sphere_intersect(ray: Ray, p: Vec3, r: f32) -> bool {
    let ap = p - ray.pos;
    let ax = ray.dir.dot(ap);
    let mag = (ap - ax * ray.dir).magnitude();
    return ax > 0.0 && mag * mag < r * r
}

fn ray_sphere_solve(ray: Ray, ps: Vec3, rs: f32) -> Option<Vec3> {
    let a = ray.dir.magnitude2();
    let b = 2.0 * (ray.pos - ps).dot(ray.dir);
    let c = (ray.pos - ps).magnitude2() - rs * rs;

    let det = b * b - 4.0 * a * c;
    if det < 0.0 {
        Option::None
    } else {
        Option::Some(
            ray.pos + ray.dir *
            (-b - det.sqrt()) / (2.0 * a)
        )
    }
}

fn frag(r: Ray) -> Color {
    {
        let ps = vec3(0.0, 0.0, -20.0);
        let rs = 4.0;
        let res = ray_sphere_solve(r, ps, rs);
        if let Some(pos) = res {
            let normal = (pos - ps).normalize();
            let proc = |x: f32| (x + 1.0) / 2.0;
            return Color::from_float(proc(normal.x), proc(normal.y), proc(normal.z), 1.0)
//            return 0xef334dff.into();
        }
    }

    // skybox
    let c1: Color = 0xad95dcff.into();
    let c2: Color = 0x1b405fff.into();
    return Color::lerp(c1, c2, 0.5 * (1.0 + r.dir.y));
}

fn apply_raytrace(img: &mut Image) {
    let zero_vec3: Vec3 = vec3(0.0, 0.0, 0.0);
    let fov: f32 = 60.0;
    let cam_size: f32 = 1.0;
    let aspect = img.width as f32 / img.height as f32;

    let back = aspect * cam_size / (Deg::tan(cgmath::Deg(fov / 2.0)));

    for x in 0 .. img.width {
        for y in 0 .. img.height {
            let ndc = |x: f32| (x - 0.5) * 2.0;
            let (x_ndc, y_ndc) = (
                ndc(x as f32 / img.width as f32),
                ndc(y as f32 / img.height as f32)
            );

            let dir = vec3(x_ndc * aspect * cam_size, y_ndc * cam_size, -back);
            let ray = Ray::new(vec3(0.0, 0.0, back), dir);

            img.put_pixel(x, y, frag(ray));
        }
    }
}
