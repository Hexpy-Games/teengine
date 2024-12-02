use glam::Mat4;
use glutin::event::{ElementState, Event, KeyboardInput, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::{ContextBuilder, PossiblyCurrent, WindowedContext};
use std::time::{Duration, Instant};

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

pub struct EngineConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub vsync: bool,
    pub fps_limit: Option<u32>,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            title: "Game".to_string(),
            width: 800,
            height: 600,
            vsync: true,
            fps_limit: None,
        }
    }
}

pub struct Engine {
    pub sprite_renderer: SpriteRenderer,
    pub input_manager: InputManager,
    pub projection: Mat4,
    pub camera: Camera,
    window_context: WindowedContext<PossiblyCurrent>,
    last_frame_time: Instant,
    delta_time: f32,
    target_fps: u32,
    frame_duration: Duration,
    fps_limit: Option<u32>,
}

impl Engine {
    pub fn new_with_game<G: Game + 'static>(
        config: EngineConfig,
        mut game: G,
    ) {
        #[cfg(debug_assertions)]
        println!("Creating event loop...");
        let event_loop = EventLoop::new();

        #[cfg(debug_assertions)]
        println!("Creating window...");
        let wb = WindowBuilder::new()
            .with_title(&config.title)
            .with_inner_size(glutin::dpi::LogicalSize::new(
                config.width as f64,
                config.height as f64,
            ))
            .with_resizable(false)
            .with_visible(true);

        #[cfg(debug_assertions)]
        println!("Creating OpenGL context...");
        let windowed_context = ContextBuilder::new()
            .with_vsync(config.vsync)
            .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 3)))
            .with_gl_profile(glutin::GlProfile::Core)
            .build_windowed(wb, &event_loop)
            .expect("Failed to create OpenGL context");

        #[cfg(debug_assertions)]
        println!("Making context current...");
        let windowed_context = unsafe {
            windowed_context
                .make_current()
                .expect("Failed to make OpenGL context current")
        };

        #[cfg(debug_assertions)]
        println!("Loading OpenGL functions...");
        gl::load_with(|symbol| {
            let proc_addr = windowed_context.get_proc_address(symbol);
            #[cfg(debug_assertions)]
            println!("Loading GL symbol: {} -> {:?}", symbol, proc_addr);
            proc_addr as *const _
        });

        #[cfg(debug_assertions)]
        println!("Setting up OpenGL state...");
        unsafe {
            gl::Viewport(0, 0, config.width as i32, config.height as i32);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }

        let projection = Mat4::orthographic_rh(
            0.0,
            config.width as f32,
            config.height as f32,
            0.0,
            -1.0,
            1.0,
        );

        #[cfg(debug_assertions)]
        println!("Creating sprite renderer...");
        let monitor = windowed_context.window().current_monitor().unwrap();
        let video_mode = monitor.video_modes().next().unwrap();
        let monitor_refresh_millihertz = video_mode.refresh_rate_millihertz();
        let monitor_refresh_rate: u32 = monitor_refresh_millihertz / 1000;

        let target_fps = config.fps_limit.unwrap_or(monitor_refresh_rate);

        let mut engine = Self {
            sprite_renderer: SpriteRenderer::new(),
            input_manager: InputManager::new(),
            projection,
            camera: Camera::new(config.width as f32, config.height as f32),
            window_context: windowed_context,
            last_frame_time: Instant::now(),
            delta_time: 1.0 / target_fps as f32,
            target_fps,
            frame_duration: Duration::from_secs_f32(1.0 / target_fps as f32),
            fps_limit: config.fps_limit,
        };

        #[cfg(debug_assertions)]
        println!("Initializing game...");
        game.init(&mut engine);

        #[cfg(debug_assertions)]
        println!("Starting game loop...");
        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            let current_time = Instant::now();
            let elapsed = current_time.duration_since(engine.last_frame_time);

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
                    if elapsed >= engine.frame_duration {
                        engine.delta_time = 1.0 / engine.target_fps as f32;
                        engine.last_frame_time = current_time;

                        engine.input_manager.update();
                        game.update(&mut engine);
                        engine.window_context.window().request_redraw();
                    } else {
                        *control_flow = ControlFlow::WaitUntil(
                            engine.last_frame_time + engine.frame_duration,
                        );
                    }
                }
                Event::RedrawRequested(_) => {
                    unsafe {
                        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
                        gl::Clear(gl::COLOR_BUFFER_BIT);
                    }
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

    pub fn get_fps_limit(&self) -> Option<u32> {
        self.fps_limit
    }
}
