extern crate gfx;
extern crate glutin;
extern crate gfx_device_gl;

use glutin::dpi::LogicalSize;

use gfx::format::Rgba8;
use gfx::format::DepthStencil;
use gfx::Device;
use gfx::handle::RenderTargetView;
use gfx::handle::DepthStencilView;

use gfx_device_gl::Resources;
use glutin::GlContext;

pub mod resource;

pub trait Game {
    fn init(&mut self, ctx: &mut GameContext);

    fn update(&mut self, ctx: &mut GameContext);
}

pub struct GfxContext {
    pub device: gfx_device_gl::Device,
    pub factory: gfx_device_gl::Factory,
    pub color_view: RenderTargetView<Resources, Rgba8>,
    pub depth_view: DepthStencilView<Resources, DepthStencil>
}

pub struct GameContext {
    pub gfx: GfxContext,
    window: glutin::GlWindow,
    events_loop: glutin::EventsLoop
}

impl GameContext {

    pub fn init() -> GameContext {
        let events_loop = glutin::EventsLoop::new();
        let window_builder = glutin::WindowBuilder::new()
            .with_title("WeAthFolD's gfx playground")
            .with_dimensions(LogicalSize::new(1280.0, 720.0));

        // GL context
        let gl_context = glutin::ContextBuilder::new();

        // Setup gfx_window_glutin
        let (window, mut device, mut factory, rtv, stv) =
            gfx_window_glutin::init::<Rgba8, DepthStencil>(window_builder, gl_context, &events_loop).unwrap();

        let ctx = GameContext {
            gfx: GfxContext {
                device,
                factory,
                color_view: rtv,
                depth_view: stv
            },
            window,
            events_loop
        };

        ctx
    }

    pub fn run_loop<C>(&mut self, cb: &mut C)
        where C: FnMut(&mut GfxContext) {
        unsafe { self.window.make_current() }.unwrap();

        let mut running = true;
        while running {
            let window = &self.window;
            let gfx = &mut self.gfx;
            self.events_loop.poll_events(|event| {
                match event {
                    glutin::Event::WindowEvent { event, .. } => match event {
                        glutin::WindowEvent::CloseRequested => running = false,
                        glutin::WindowEvent::Resized(new_size) => {
                            let dpi_factor = window.get_hidpi_factor();
                            window.resize(new_size.to_physical(dpi_factor));
                            // Here resized new views will be created
                            gfx_window_glutin::update_views(
                                &window, &mut gfx.color_view, &mut gfx.depth_view);
                        },
                        _ => ()
                    },
                    _ => ()
                }
            });

            cb(gfx);

            // Swap buffers
            window.swap_buffers().unwrap();
            gfx.device.cleanup();
        }
    }

}

