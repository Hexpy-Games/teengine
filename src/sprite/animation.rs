use std::time::{Duration, Instant};

pub trait AnimationProvider {
    fn get_current_frame(&mut self) -> Option<usize>;
}

#[derive(Clone)]
pub struct AnimationSequence {
    pub name: String,
    frames: Vec<usize>,
    frame_duration: Duration,
    current_frame_index: usize,
    repeat: bool,
    last_update: Instant,
}

impl AnimationSequence {
    pub fn new(
        name: String,
        frames: Vec<usize>,
        frame_duration_ms: u64,
        repeat: bool,
    ) -> Self {
        Self {
            name,
            frames,
            frame_duration: Duration::from_millis(frame_duration_ms),
            current_frame_index: 0,
            repeat,
            last_update: Instant::now(),
        }
    }

    pub fn reset(&mut self) {
        self.current_frame_index = 0;
        self.last_update = Instant::now();
    }
}

impl AnimationProvider for AnimationSequence {
    fn get_current_frame(&mut self) -> Option<usize> {
        if self.current_frame_index >= self.frames.len() {
            if self.repeat {
                self.current_frame_index = 0;
            } else {
                return None;
            }
        }

        if self.last_update.elapsed() >= self.frame_duration {
            self.last_update = Instant::now();
            let frame = self.frames[self.current_frame_index];
            self.current_frame_index += 1;
            Some(frame)
        } else {
            Some(self.frames[self.current_frame_index])
        }
    }
}
