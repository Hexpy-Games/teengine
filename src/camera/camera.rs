use glam::{Mat4, Vec2};

pub struct Camera {
    // current position of camera
    position: Vec2,
    // size of viewport
    viewport_size: Vec2,
    // interpolation speed (0.0 ~ 1.0, 1.0 is fastest)
    lerp_speed: f32,
    // world size limit
    world_bounds: Option<(Vec2, Vec2)>, // (min, max)
    // zoom level
    zoom: f32,
}

impl Camera {
    pub fn new(
        viewport_width: f32,
        viewport_height: f32,
    ) -> Self {
        Self {
            position: Vec2::ZERO,
            viewport_size: Vec2::new(viewport_width, viewport_height),
            lerp_speed: 0.1,
            world_bounds: None,
            zoom: 1.0,
        }
    }

    /// set world bounds
    pub fn set_world_bounds(
        &mut self,
        min: Vec2,
        max: Vec2,
    ) {
        self.world_bounds = Some((min, max));
    }

    /// follow target smoothly
    pub fn follow_target(
        &mut self,
        target_pos: Vec2,
        delta_time: f32,
    ) {
        // calculate camera position so that the target is centered on the screen
        let target_camera_pos =
            target_pos - self.viewport_size * 0.5 / self.zoom;

        // implement smooth movement with linear interpolation (LERP)
        let lerp_factor = if self.lerp_speed >= 0.99 {
            1.0 // instant move
        } else {
            // apply lerp_speed more strongly to exponential interpolation
            1.0 - (-self.lerp_speed * 15.0 * delta_time * 60.0).exp()
        };

        let new_pos = self.position.lerp(target_camera_pos, lerp_factor);

        // apply world bounds limit
        self.position = if let Some((min, max)) = self.world_bounds {
            Vec2::new(
                new_pos
                    .x
                    .clamp(min.x, max.x - self.viewport_size.x / self.zoom),
                new_pos
                    .y
                    .clamp(min.y, max.y - self.viewport_size.y / self.zoom),
            )
        } else {
            new_pos
        };
    }

    /// return projection matrix of camera
    pub fn get_projection_matrix(&self) -> Mat4 {
        Mat4::orthographic_rh(
            self.position.x * self.zoom,
            (self.position.x + self.viewport_size.x) * self.zoom,
            (self.position.y + self.viewport_size.y) * self.zoom,
            self.position.y * self.zoom,
            -1.0,
            1.0,
        )
    }

    /// set zoom level
    pub fn set_zoom(
        &mut self,
        zoom: f32,
    ) {
        self.zoom = zoom.clamp(0.1, 10.0);
    }

    /// set interpolation speed
    pub fn set_lerp_speed(
        &mut self,
        speed: f32,
    ) {
        self.lerp_speed = speed.clamp(0.0, 1.0);
    }

    /// convert world coordinates to screen coordinates
    pub fn world_to_screen(
        &self,
        world_pos: Vec2,
    ) -> Vec2 {
        (world_pos - self.position) * self.zoom
    }

    /// convert screen coordinates to world coordinates
    pub fn screen_to_world(
        &self,
        screen_pos: Vec2,
    ) -> Vec2 {
        screen_pos / self.zoom + self.position
    }

    pub fn get_world_bounds(&self) -> Option<(Vec2, Vec2)> {
        self.world_bounds
    }
}
