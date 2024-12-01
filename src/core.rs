use glam::Mat4;
use glutin::event::{ElementState, Event, KeyboardInput, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;
use std::time::Instant;

use crate::input::input_manager::InputManager;
use crate::sprite::sprite_renderer::SpriteRenderer;
use crate::Camera;

pub trait Game {
    fn init(
        &mut self,
        engine: &mut Engine,
    );
    fn update(
        &mut self,
        engine: &mut Engine,
    );
    fn render(
        &mut self,
        engine: &mut Engine,
    );
}

pub struct Engine {
    pub sprite_renderer: SpriteRenderer,
    pub input_manager: InputManager,
    pub projection: Mat4,
    pub camera: Camera,
    window_context: glutin::WindowedContext<glutin::PossiblyCurrent>,
    last_frame_time: Instant,
    delta_time: f32,
}

impl Engine {
    pub fn new_with_game<G: Game + 'static>(
        title: &str,
        width: u32,
        height: u32,
        vsync: bool,
        mut game: G,
    ) {
        println!("Creating event loop...");
        let event_loop = EventLoop::new();

        println!("Creating window...");
        let wb = WindowBuilder::new()
            .with_title(title)
            .with_inner_size(glutin::dpi::LogicalSize::new(
                width as f64,
                height as f64,
            ))
            .with_resizable(false)
            .with_visible(true);

        println!("Creating OpenGL context...");
        let windowed_context = ContextBuilder::new()
            .with_vsync(vsync)
            .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 3)))
            .with_gl_profile(glutin::GlProfile::Core)
            .build_windowed(wb, &event_loop)
            .expect("Failed to create OpenGL context");

        println!("Making context current...");
        let windowed_context = unsafe {
            windowed_context
                .make_current()
                .expect("Failed to make OpenGL context current")
        };

        println!("Loading OpenGL functions...");
        gl::load_with(|symbol| {
            let proc_addr = windowed_context.get_proc_address(symbol);
            println!("Loading GL symbol: {} -> {:?}", symbol, proc_addr);
            proc_addr as *const _
        });

        println!("Setting up OpenGL state...");
        unsafe {
            gl::Viewport(0, 0, width as i32, height as i32);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }

        let projection = Mat4::orthographic_rh(
            0.0,
            width as f32,
            height as f32,
            0.0,
            -1.0,
            1.0,
        );

        println!("Creating sprite renderer...");
        let mut engine = Self {
            sprite_renderer: SpriteRenderer::new(),
            input_manager: InputManager::new(),
            projection,
            camera: Camera::new(width as f32, height as f32),
            window_context: windowed_context,
            last_frame_time: Instant::now(),
            delta_time: 0.0,
        };

        println!("Initializing game...");
        game.init(&mut engine);

        println!("Starting game loop...");
        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            let current_time = Instant::now();
            engine.delta_time = current_time
                .duration_since(engine.last_frame_time)
                .as_secs_f32();
            engine.last_frame_time = current_time;

            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit
                    }
                    WindowEvent::Resized(physical_size) => {
                        engine.window_context.resize(physical_size);
                        unsafe {
                            gl::Viewport(
                                0,
                                0,
                                physical_size.width as i32,
                                physical_size.height as i32,
                            );
                        }
                        engine.projection = Mat4::orthographic_rh(
                            0.0,
                            physical_size.width as f32,
                            physical_size.height as f32,
                            0.0,
                            -1.0,
                            1.0,
                        );
                    }
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(keycode),
                                state,
                                ..
                            },
                        ..
                    } => {
                        let pressed = state == ElementState::Pressed;
                        engine
                            .input_manager
                            .process_keyboard_input(keycode, pressed);
                    }
                    _ => (),
                },
                Event::MainEventsCleared => {
                    engine.input_manager.update();
                    game.update(&mut engine);
                    engine.window_context.window().request_redraw();
                }
                Event::RedrawRequested(_) => {
                    unsafe {
                        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
                        gl::Clear(gl::COLOR_BUFFER_BIT);
                    }
                    let projection = engine.camera.get_projection_matrix();
                    game.render(&mut engine);
                    engine.window_context.swap_buffers().unwrap();
                }
                _ => (),
            }
        });
    }

    pub fn delta_time(&self) -> f32 {
        self.delta_time
    }
}
