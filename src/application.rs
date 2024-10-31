use std::num::NonZeroU32;

use glow::*;
use glutin::{
    config::{Config, ConfigTemplateBuilder},
    context::{ContextApi, ContextAttributesBuilder, PossiblyCurrentContext, Version},
    display::{GetGlDisplay, GlDisplay},
    prelude::NotCurrentGlContext,
    surface::{GlSurface, Surface, SurfaceAttributesBuilder, SwapInterval, WindowSurface},
};
use glutin_winit::DisplayBuilder;

use winit::{
    self,
    application::ApplicationHandler,
    event::{StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    raw_window_handle::HasWindowHandle,
    window::{Window, WindowAttributes, WindowId},
};

pub struct Application {
    window: Window,
    gl_config: Config,
    renderer: Option<Renderer>,
    exit_state: Result<(), Box<dyn std::error::Error>>,
}

impl Application {
    pub fn new(event_loop: &EventLoop<()>) -> Result<Self, Box<dyn std::error::Error>> {
        let (window, gl_config) = Self::create_window(event_loop)?;
        let (context, surface, glow_context) = Self::create_gl_context(&window, &gl_config)?;

        unsafe {
            let gl = glow::Context::from_loader_function_cstr(|s| {
                context.display().get_proc_address(s).cast()
            });

            let version = gl.get_parameter_string(glow::VERSION);
            println!("OpenGL version {}", version);

            let buffer = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(buffer));

            let vertices: Vec<f32> = vec![-0.5, -0.5, 0.0, 0.5, 0.5, -0.5];
            let data = bytemuck::cast_slice(&vertices);
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, data, glow::STATIC_DRAW);
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, size_of::<f32>() as i32 * 2, 0);
        }

        let app = Self {
            window,
            gl_config,
            renderer: None,
            exit_state: Ok(()),
        };

        Ok(app)
    }

    fn create_window(
        event_loop: &EventLoop<()>,
    ) -> Result<(Window, Config), Box<dyn std::error::Error>> {
        // Build winit Window
        let window_attributes = WindowAttributes::default()
            .with_title("OpenGL window")
            .with_transparent(true);

        let template_builder = ConfigTemplateBuilder::new();

        let (window, gl_config) = DisplayBuilder::new()
            .with_window_attributes(Some(window_attributes))
            .build(event_loop, template_builder, |mut configs| {
                configs.next().unwrap()
            })
            .expect("Failed to create OpenGL window");

        let window = window.unwrap(); // Remove Option<Window>

        Ok((window, gl_config))
    }

    fn create_gl_context(
        window: &Window,
        gl_config: &Config,
    ) -> Result<
        (
            PossiblyCurrentContext,
            Surface<WindowSurface>,
            glow::Context,
        ),
        Box<dyn std::error::Error>,
    > {
        let raw_window_handle = window.window_handle().unwrap().as_raw();

        let context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(Some(Version::new(4, 6)))) // Usamos Version::new(4, 6)
            .build(Some(raw_window_handle));

        // Since glutin by default tries to create OpenGL core context, which may not be
        // present we should try gles.
        let fallback_context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::Gles(None))
            .build(Some(raw_window_handle));

        // There are also some old devices that support neither modern OpenGL nor GLES.
        // To support these we can try and create a 2.1 context.
        let legacy_context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(Some(Version::new(2, 1))))
            .build(Some(raw_window_handle));

        let gl_display = gl_config.display();

        // Create the context with the highest version of OpenGL that is supported.
        let context = unsafe {
            gl_display
                .create_context(&gl_config, &context_attributes)
                .unwrap_or_else(|_| {
                    gl_display
                        .create_context(&gl_config, &fallback_context_attributes)
                        .unwrap_or_else(|_| {
                            gl_display
                                .create_context(&gl_config, &legacy_context_attributes)
                                .expect("failed to create context")
                        })
                })
        };

        let surface_attribs = SurfaceAttributesBuilder::<WindowSurface>::new()
            .with_srgb(Some(true))
            .build(
                window.window_handle().unwrap().as_raw(),
                NonZeroU32::new(1024).unwrap(),
                NonZeroU32::new(768).unwrap(),
            );
        let surface = unsafe {
            gl_display
                .create_window_surface(&gl_config, &surface_attribs)
                .expect("Failed to create OpenGL surface")
        };

        let context = context
            .make_current(&surface)
            .expect("Failed to make OpenGL context current");

        // Set the swap interval to 1 to enable vsync.
        surface
            .set_swap_interval(&context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()))
            .expect("Failed to set swap interval");

        let glow_context = unsafe {
            glow::Context::from_loader_function_cstr(|s| {
                context.display().get_proc_address(s).cast()
            })
        };

        Ok((context, surface, glow_context))
    }
}

impl ApplicationHandler for Application {
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                self.exit_state = Err("Window closed".into());
                event_loop.exit();
            }
            _ => (),
        }
    }

    fn new_events(&mut self, _: &ActiveEventLoop, _: StartCause) {}

    fn resumed(&mut self, _: &ActiveEventLoop) {}
}

pub struct Renderer {}
