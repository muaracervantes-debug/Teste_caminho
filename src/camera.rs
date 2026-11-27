use std::collections::HashSet;

use glam::{Mat4, Vec3};
use winit::{
    event::{ElementState, KeyEvent, MouseScrollDelta},
    keyboard::{KeyCode, PhysicalKey},
};

const MIN_DISTANCE: f32 = 4.0;
const MAX_DISTANCE: f32 = 100.0;
const DEFAULT_DISTANCE: f32 = 24.0;
const DEFAULT_PITCH: f32 = 55.0_f32.to_radians();
const MOVEMENT_SPEED: f32 = 5.5;
const ROTATION_SPEED: f32 = 1.4;
const ZOOM_SENSITIVITY: f32 = 0.12;

pub struct Camera {
    target: Vec3,
    yaw: f32,
    pitch: f32,
    distance: f32,
    pressed_keys: HashSet<KeyCode>,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            target: Vec3::ZERO,
            yaw: 45.0_f32.to_radians(),
            pitch: DEFAULT_PITCH,
            distance: DEFAULT_DISTANCE,
            pressed_keys: HashSet::new(),
        }
    }

    pub fn process_keyboard(&mut self, event: &KeyEvent) {
        let PhysicalKey::Code(key) = event.physical_key else {
            return;
        };

        match event.state {
            ElementState::Pressed => {
                self.pressed_keys.insert(key);
            }
            ElementState::Released => {
                self.pressed_keys.remove(&key);
            }
        }
    }

    pub fn process_mouse_wheel(&mut self, delta: &MouseScrollDelta) -> bool {
        let scroll = match delta {
            MouseScrollDelta::LineDelta(_, y) => *y,
            MouseScrollDelta::PixelDelta(position) => (position.y / 100.0) as f32,
        };

        if scroll.abs() < f32::EPSILON {
            return false;
        }

        self.apply_zoom(scroll);
        true
    }

    pub fn clear_pressed_keys(&mut self) {
        self.pressed_keys.clear();
    }

    pub fn update(&mut self, delta_seconds: f32) -> bool {
        let delta_seconds = delta_seconds.clamp(0.0, 0.1);
        if delta_seconds <= 0.0 {
            return false;
        }

        let mut changed = false;
        let mut rotation = 0.0;

        if self.is_pressed(KeyCode::KeyQ) {
            rotation -= ROTATION_SPEED * delta_seconds;
        }
        if self.is_pressed(KeyCode::KeyE) {
            rotation += ROTATION_SPEED * delta_seconds;
        }

        if rotation != 0.0 {
            self.yaw += rotation;
            changed = true;
        }

        let forward = Vec3::new(-self.yaw.sin(), 0.0, -self.yaw.cos());
        let right = Vec3::new(self.yaw.cos(), 0.0, -self.yaw.sin());

        let mut movement = Vec3::ZERO;

        if self.is_pressed(KeyCode::KeyW) || self.is_pressed(KeyCode::ArrowUp) {
            movement += forward;
        }
        if self.is_pressed(KeyCode::KeyS) || self.is_pressed(KeyCode::ArrowDown) {
            movement -= forward;
        }
        if self.is_pressed(KeyCode::KeyD) || self.is_pressed(KeyCode::ArrowRight) {
            movement += right;
        }
        if self.is_pressed(KeyCode::KeyA) || self.is_pressed(KeyCode::ArrowLeft) {
            movement -= right;
        }

        if movement.length_squared() > 0.0 {
            movement = movement.normalize();
            let speed = MOVEMENT_SPEED * (self.distance / DEFAULT_DISTANCE).sqrt();
            self.target += movement * speed * delta_seconds;
            changed = true;
        }

        changed
    }

    pub fn view_projection(&self, width: u32, height: u32) -> Mat4 {
        let aspect = width.max(1) as f32 / height.max(1) as f32;
        let projection = Mat4::perspective_rh(45.0_f32.to_radians(), aspect, 0.1, 1000.0);

        let horizontal_distance = self.distance * self.pitch.cos();
        let eye = self.target
            + Vec3::new(
                self.yaw.sin() * horizontal_distance,
                self.pitch.sin() * self.distance,
                self.yaw.cos() * horizontal_distance,
            );

        let view = Mat4::look_at_rh(eye, self.target, Vec3::Y);
        projection * view
    }

    fn apply_zoom(&mut self, scroll: f32) {
        self.distance = (self.distance * (-scroll * ZOOM_SENSITIVITY).exp())
            .clamp(MIN_DISTANCE, MAX_DISTANCE);
    }

    fn is_pressed(&self, key: KeyCode) -> bool {
        self.pressed_keys.contains(&key)
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn view_projection_is_finite() {
        let camera = Camera::new();
        let values = camera.view_projection(1280, 720).to_cols_array();

        assert!(values.iter().all(|value| value.is_finite()));
    }

    #[test]
    fn zoom_stays_inside_supported_limits() {
        let mut camera = Camera::new();

        camera.apply_zoom(1000.0);
        assert_eq!(camera.distance, MIN_DISTANCE);

        camera.apply_zoom(-1000.0);
        assert_eq!(camera.distance, MAX_DISTANCE);
    }
}
