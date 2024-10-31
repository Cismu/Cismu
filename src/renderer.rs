use glow::*;

use glutin::{
    context::PossiblyCurrentContext,
    display::{GetGlDisplay, GlDisplay},
    surface::{GlSurface, Surface, WindowSurface},
};

// use indoc::indoc;

use std::fs::File;
use std::io::{self, Read};

pub struct Renderer {
    gl: glow::Context,
    context: PossiblyCurrentContext,
    surface: Surface<WindowSurface>,
    program: Option<NativeProgram>,
}

impl Renderer {
    pub fn new(context: PossiblyCurrentContext, surface: Surface<WindowSurface>) -> Self {
        let gl = unsafe {
            let gl = glow::Context::from_loader_function_cstr(|s| {
                context.display().get_proc_address(s).cast()
            });
            let version = gl.get_parameter_string(glow::VERSION);
            println!("OpenGL version {}", version);

            gl
        };

        let mut renderer = Self {
            gl,
            context,
            surface,
            program: None,
        };

        renderer.draw();

        renderer
    }

    fn draw(&mut self) {
        unsafe {
            let gl = &self.gl;

            let vao = gl.create_vertex_array().unwrap();
            gl.bind_vertex_array(Some(vao));

            let buffer = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(buffer));

            let vertices: Vec<f32> = vec![
                -0.5, -0.5,
                0.5, -0.5,
                0.5, 0.5,

                0.5, 0.5,
                -0.5, 0.5,
                -0.5, -0.5,
            ];

            let data = bytemuck::cast_slice(&vertices);
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, data, glow::STATIC_DRAW);

            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, size_of::<f32>() as i32 * 2, 0);

            let (vertex_shader_src, fragment_shader_src) =
                self.load_shaders("./assets/shaders/basic.shader").unwrap();

            let program = gl.create_program().expect("Cannot create program");
            let vs = self.compile_shader(&gl, vertex_shader_src.as_str(), glow::VERTEX_SHADER);
            let fs = self.compile_shader(&gl, fragment_shader_src.as_str(), glow::FRAGMENT_SHADER);

            gl.attach_shader(program, vs);
            gl.attach_shader(program, fs);
            gl.link_program(program);

            gl.delete_shader(vs);
            gl.delete_shader(fs);

            self.program = Some(program);
        }
    }

    unsafe fn compile_shader(
        &self,
        gl: &glow::Context,
        shader_src: &str,
        shader_type: u32,
    ) -> NativeShader {
        let vs = gl
            .create_shader(shader_type)
            .expect("Cannot create vertex shader");
        gl.shader_source(vs, shader_src);
        gl.compile_shader(vs);

        let status = gl.get_shader_compile_status(vs);
        if !status {
            let log = gl.get_shader_info_log(vs);
            println!("Vertex shader compilation failed: {}", log);
        }

        return vs;
    }

    pub unsafe fn render(&self) {
        let gl = &self.gl;

        gl.clear_color(0.05, 0.05, 0.1, 1.0); // Background color
        gl.clear(glow::COLOR_BUFFER_BIT);
        gl.use_program(self.program);
        // gl.bind_vertex_array(Some(self.vertex_array));
        gl.draw_arrays(glow::TRIANGLES, 0, 6);

        self.surface
            .swap_buffers(&self.context)
            .expect("Failed to swap buffers");
    }

    pub fn cleanup(&self) {
        unsafe {
            if let Some(program) = self.program {
                self.gl.delete_program(program);
            }

            // gl.delete_vertex_array(self.vertex_array);
        }
    }

    fn load_shaders(&self, file_path: &str) -> io::Result<(String, String)> {
        let mut file = File::open(file_path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        let mut vert_shader = String::new();
        let mut frag_shader = String::new();
        let mut active_shader = None;

        for line in content.lines() {
            if line.starts_with("#shader") {
                if line.contains("vertex") {
                    active_shader = Some(&mut vert_shader);
                } else if line.contains("fragment") {
                    active_shader = Some(&mut frag_shader);
                }
            } else if let Some(shader) = &mut active_shader {
                shader.push_str(line);
                shader.push('\n');
            }
        }

        Ok((vert_shader, frag_shader))
    }
}
