use std::{num::NonZeroU32, time::Instant};

mod utils;

use bytemuck::cast_slice;
use glow::*;
use glutin::display::{GetGlDisplay, GlDisplay};
use glutin::surface::GlSurface;
use winit::application::ApplicationHandler;
use winit::event::{Event, WindowEvent};

struct MyApp {
    imgui_context: imgui::Context,
    winit_platform: imgui_winit_support::WinitPlatform,
    ig_renderer: imgui_glow_renderer::AutoRenderer,
    last_frame_time: Instant,
    surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
    context: glutin::context::PossiblyCurrentContext,
    window: winit::window::Window,
}

impl ApplicationHandler for MyApp {
    fn new_events(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _cause: winit::event::StartCause,
    ) {
        self.update_frame_time();
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::RedrawRequested => {
                self.render_scene();
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                self.resize_surface(new_size);
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        self.prepare_for_rendering();
    }

    fn exiting(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        self.cleanup_rendering();
    }

    fn resumed(&mut self, _: &winit::event_loop::ActiveEventLoop) {}
}

impl MyApp {
    fn new(
        imgui_context: imgui::Context,
        winit_platform: imgui_winit_support::WinitPlatform,
        ig_renderer: imgui_glow_renderer::AutoRenderer,
        surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
        context: glutin::context::PossiblyCurrentContext,
        window: winit::window::Window,
    ) -> Self {
        Self {
            imgui_context,
            winit_platform,
            ig_renderer,
            last_frame_time: Instant::now(),
            surface,
            context,
            window,
        }
    }

    fn update_frame_time(&mut self) {
        let current_frame_time = Instant::now();
        let delta_time = current_frame_time.duration_since(self.last_frame_time);
        self.imgui_context.io_mut().update_delta_time(delta_time);
        self.last_frame_time = current_frame_time;
    }

    fn prepare_for_rendering(&mut self) {
        // Prepara el contexto de imgui para el próximo cuadro.
        self.winit_platform
            .prepare_frame(self.imgui_context.io_mut(), &self.window)
            .unwrap();
    }

    fn render_scene(&mut self) {
        unsafe {
            let gl = glow::Context::from_loader_function_cstr(|s| {
                self.context.display().get_proc_address(s).cast()
            });

            let version = gl.get_parameter_string(glow::VERSION);
            println!("OpenGL version {}", version);

            // Define los datos de los vértices en f32
            let vertices: Vec<f32> = vec![-0.5, -0.5, 0.0, 0.5, 0.5, -0.5];

            gl.clear(glow::COLOR_BUFFER_BIT);

            // Crea el buffer en OpenGL
            let buffer = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(buffer));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, cast_slice(&vertices), glow::STATIC_DRAW);
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(
                0,
                2,
                glow::FLOAT,
                false,
                std::mem::size_of::<f32>() as i32 * 2,
                0,
            );

            gl.draw_arrays(glow::TRIANGLES, 0, 3);
        }

        self.surface
            .swap_buffers(&self.context)
            .expect("Failed to swap buffers");
    }

    fn resize_surface(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.surface.resize(
                &self.context,
                NonZeroU32::new(new_size.width).unwrap(),
                NonZeroU32::new(new_size.height).unwrap(),
            );
        }
    }

    fn cleanup_rendering(&mut self) {
        // Aquí puedes realizar tareas de limpieza, como liberar recursos de `imgui`.
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (event_loop, window, surface, context) = utils::create_window("Hello, triangle!");
    let (winit_platform, imgui_context, ig_renderer) = initialize_renderers(&window, &context)?;

    let mut application = MyApp::new(
        imgui_context,
        winit_platform,
        ig_renderer,
        surface,
        context,
        window,
    );

    event_loop.run_app(&mut application)?;

    Ok(())
}

fn initialize_renderers(
    window: &winit::window::Window,
    context: &glutin::context::PossiblyCurrentContext,
) -> Result<
    (
        imgui_winit_support::WinitPlatform,
        imgui::Context,
        imgui_glow_renderer::AutoRenderer,
    ),
    Box<dyn std::error::Error>,
> {
    let (winit_platform, mut imgui_context) = utils::imgui_init(&window);
    let gl = utils::glow_context(&context);
    let ig_renderer = imgui_glow_renderer::AutoRenderer::new(gl, &mut imgui_context)
        .expect("failed to create renderer");

    Ok((winit_platform, imgui_context, ig_renderer))
}
