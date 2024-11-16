use glam::{Mat4, Vec2};
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;
use utils::color_key_util::ColorKey;
use std::path::Path;
use std::time::{Duration, Instant};

mod shaders;
mod sprite;
mod sprite_renderer;
mod texture;
mod utils;

use sprite::{Rect, Sprite};
use sprite_renderer::SpriteRenderer;
use texture::Texture;

struct Game {
    sprite_renderer: SpriteRenderer,
    sprite: Sprite,
    projection: Mat4,
    last_frame_change: Instant,
    frame_duration: Duration,
}

impl Game {
    fn new(width: u32, height: u32) -> Self {
        let sprite_renderer = SpriteRenderer::new();
        let texture = Texture::new(Path::new("assets/sprite.png")).expect("Failed to load texture");
        let sprite = Sprite::new(
            texture,
            Vec2::new(0.0, 0.0),
            0.0,
            Rect::new(256.0, 256.0),
            Rect::new(1024.0, 1024.0),
            50.0,
            Some("#C6C6C4"),
        );
        let projection = Mat4::orthographic_rh(0.0, width as f32, height as f32, 0.0, -1.0, 1.0);

        Self {
            sprite_renderer,
            sprite,
            projection,
            last_frame_change: Instant::now(),
            frame_duration: Duration::from_millis(120),
        }
    }

    fn update(&mut self) {
        // 프레임 업데이트
        if self.last_frame_change.elapsed() >= self.frame_duration {
            let next_frame = (self.sprite.get_current_frame() + 1) % 4; // 0~4 프레임 순환
            self.sprite.update_frame(next_frame);
            self.last_frame_change = Instant::now();
        }
    }

    fn render(&self) {
        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        self.sprite_renderer
            .draw_sprite(&self.sprite, &self.projection);
    }
}

fn main() {
    let el = EventLoop::new();
    let wb = WindowBuilder::new().with_title("Teengine!");
    let windowed_context = ContextBuilder::new()
        .with_vsync(true)
        .build_windowed(wb, &el)
        .unwrap();

    let windowed_context = unsafe { windowed_context.make_current().unwrap() };

    gl::load_with(|symbol| windowed_context.get_proc_address(symbol) as *const _);

    let window = windowed_context.window();
    let size = window.inner_size();
    let mut game = Game::new(size.width, size.height);

    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::LoopDestroyed => return,
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(physical_size) => {
                    windowed_context.resize(physical_size);
                    game.projection = Mat4::orthographic_rh(
                        0.0,
                        physical_size.width as f32,
                        physical_size.height as f32,
                        0.0,
                        -1.0,
                        1.0,
                    );
                }
                _ => (),
            },
            Event::MainEventsCleared => {
                game.update();
                windowed_context.window().request_redraw();
            }
            Event::RedrawRequested(_) => {
                game.render();
                windowed_context.swap_buffers().unwrap();
            }
            _ => (),
        }
    });
}
