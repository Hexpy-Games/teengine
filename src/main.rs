use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::{ContextBuilder, PossiblyCurrent, WindowedContext};
use glam::{Mat4, Vec3};
use std::time::{Instant, Duration};
use glutin::dpi::PhysicalSize;

mod shaders;

struct Renderer {
    program: gl::types::GLuint,
    vao: gl::types::GLuint,
    vbo: gl::types::GLuint,
    start_time: Instant,
    last_update: Instant,
}

impl Renderer {
    fn new() -> Self {
        let vertex_shader = shaders::compile_shader(gl::VERTEX_SHADER, shaders::VERTEX_SHADER_SOURCE);
        let fragment_shader = shaders::compile_shader(gl::FRAGMENT_SHADER, shaders::FRAGMENT_SHADER_SOURCE);
        let program = shaders::link_program(vertex_shader, fragment_shader);

        let mut vao = 0;
        let mut vbo = 0;

        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);

            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

            let vertices: [f32; 15] = [
                // positions    // colors
                0.5, -0.5,     1.0, 0.0, 0.0,   // bottom right
                -0.5, -0.5,    0.0, 1.0, 0.0,   // bottom left
                0.0,  0.5,     0.0, 0.0, 1.0    // top
            ];

            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                vertices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 5 * std::mem::size_of::<f32>() as gl::types::GLsizei, std::ptr::null());
            gl::EnableVertexAttribArray(0);

            gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, 5 * std::mem::size_of::<f32>() as gl::types::GLsizei, (2 * std::mem::size_of::<f32>()) as *const () as *const _);
            gl::EnableVertexAttribArray(1);
        }

        let now = Instant::now();

        Self { program, vao, vbo, start_time: now, last_update: now }
    }

    fn update(&mut self) {
        let now = Instant::now();
        let dt = now - self.last_update;
        self.last_update = now;
    }

    fn render(&self) {
        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::UseProgram(self.program);

            let elapsed = self.start_time.elapsed().as_secs_f32();
            let transform = Mat4::from_rotation_z(elapsed);
            let transform_loc = gl::GetUniformLocation(self.program, b"transform\0".as_ptr() as *const _);
            gl::UniformMatrix4fv(transform_loc, 1, gl::FALSE, transform.to_cols_array().as_ptr());

            gl::BindVertexArray(self.vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }
    }
}

struct Options {
    pub use_vsync: bool,
}

fn main() {
    let options = Options { use_vsync: true };
    let el = EventLoop::new();
    let wb = WindowBuilder::new().with_title("Rust Game Engine");
    let windowed_context = ContextBuilder::new()
        .with_vsync(options.use_vsync)
        .build_windowed(wb, &el)
        .unwrap();

    let windowed_context = unsafe { windowed_context.make_current().unwrap() };

    gl::load_with(|symbol| windowed_context.get_proc_address(symbol) as *const _);

    let mut renderer = Renderer::new();

    let mut last_frame_time = Instant::now();
    let monitor = windowed_context.window().current_monitor().unwrap();
    let video_mode = monitor.video_modes().next().unwrap();
    let refresh_rate = video_mode.refresh_rate_millihertz() / 1000;
    let frame_duration = Duration::from_secs_f32(1.0 / refresh_rate as f32);

    print!("Display refresh rate: {} Hz\n", refresh_rate);

    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::LoopDestroyed => return,
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(physical_size) => {
                    windowed_context.resize(physical_size);
                }
                _ => (),
            },
            Event::MainEventsCleared => {
                let now = Instant::now();
                if now - last_frame_time >= frame_duration {
                    renderer.update();
                    windowed_context.window().request_redraw();
                    last_frame_time = now;
                }
            }
            Event::RedrawRequested(_) => {
                renderer.render();
                windowed_context.swap_buffers().unwrap();
            }
            _ => (),
        }
    });
}