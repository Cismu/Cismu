use std::time::{Duration, Instant};

use glutin::{
    config::{Config, ConfigTemplateBuilder},
    context::{ContextApi, ContextAttributesBuilder, PossiblyCurrentContext, Version},
    display::{GetGlDisplay, GlDisplay},
    prelude::NotCurrentGlContext,
    surface::{GlSurface, Surface, SurfaceAttributesBuilder, SwapInterval, WindowSurface},
};
use glutin_winit::DisplayBuilder;
use std::num::NonZeroU32;

use winit::{
    self,
    application::ApplicationHandler,
    event::{StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    raw_window_handle::HasWindowHandle,
    window::{Window, WindowAttributes, WindowId},
};

use crate::renderer::Renderer;

// Define la frecuencia de actualización deseada (por ejemplo, 60 FPS)
const FPS: u32 = 60;
const FRAME_DURATION: Duration = Duration::new(0, 1_000_000_000 / FPS);

pub struct Application {
    window: Window,
    gl_config: Config,
    renderer: Option<Renderer>,
    exit_state: Result<(), Box<dyn std::error::Error>>,
}

impl Application {
    pub fn new(event_loop: &EventLoop<()>) -> Result<Self, Box<dyn std::error::Error>> {
        let (window, gl_config) = Self::create_window(event_loop)?;

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
    ) -> Result<(PossiblyCurrentContext, Surface<WindowSurface>), Box<dyn std::error::Error>> {
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

        Ok((context, surface))
    }
}

impl ApplicationHandler for Application {
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::RedrawRequested => unsafe {
                if let Some(renderer) = &self.renderer {
                    renderer.render();

                    // Calcula el tiempo para el próximo fotograma
                    let next_frame_time = Instant::now() + FRAME_DURATION;
                    event_loop.set_control_flow(ControlFlow::WaitUntil(next_frame_time));
                }
            },
            WindowEvent::CloseRequested => {
                if let Some(renderer) = &self.renderer {
                    renderer.cleanup();
                    self.renderer = None;
                }

                self.exit_state = Err("Window closed".into());
                event_loop.exit();
            }
            _ => (),
        }
    }

    fn new_events(&mut self, _: &ActiveEventLoop, _: StartCause) {}

    // This function is called when the application is resumed from a suspended state. Or when the appplication is started.
    // This is a good place to initialize the renderer (graphics context) and other resources.
    fn resumed(&mut self, _: &ActiveEventLoop) {
        let ctx = Self::create_gl_context(&self.window, &self.gl_config).unwrap();
        let (context, surface) = ctx;

        self.renderer = Some(Renderer::new(context, surface));
    }
}
