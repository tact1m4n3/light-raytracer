use light_raytracer::Camera;
use winit::{
    event::{ElementState, MouseButton},
    keyboard::{KeyCode, PhysicalKey},
    window::{CursorGrabMode, Window},
};

pub struct CameraController {
    speed: f32,
    sensitivity: f32,
    enabled: bool,
    key_inputs: [bool; 6],
    mouse_delta: glam::Vec2,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            speed,
            sensitivity,
            enabled: false,
            key_inputs: [false; 6],
            mouse_delta: glam::vec2(0.0, 0.0),
        }
    }

    pub fn on_mouse_event(&mut self, button: MouseButton, state: ElementState) {
        if let MouseButton::Left = button {
            self.enabled = state.is_pressed();
        }
    }

    pub fn on_key_event(&mut self, physical_key: PhysicalKey, state: ElementState) {
        match physical_key {
            PhysicalKey::Code(KeyCode::KeyW) | PhysicalKey::Code(KeyCode::ArrowUp) => {
                self.key_inputs[0] = state.is_pressed();
            }
            PhysicalKey::Code(KeyCode::KeyS) | PhysicalKey::Code(KeyCode::ArrowDown) => {
                self.key_inputs[1] = state.is_pressed();
            }
            PhysicalKey::Code(KeyCode::KeyA) | PhysicalKey::Code(KeyCode::ArrowLeft) => {
                self.key_inputs[2] = state.is_pressed();
            }
            PhysicalKey::Code(KeyCode::KeyD) | PhysicalKey::Code(KeyCode::ArrowRight) => {
                self.key_inputs[3] = state.is_pressed();
            }
            PhysicalKey::Code(KeyCode::KeyQ) | PhysicalKey::Code(KeyCode::ShiftLeft) => {
                self.key_inputs[4] = state.is_pressed();
            }
            PhysicalKey::Code(KeyCode::KeyE) | PhysicalKey::Code(KeyCode::Space) => {
                self.key_inputs[5] = state.is_pressed();
            }
            _ => {}
        }
    }

    pub fn on_mouse_motion(&mut self, delta: glam::Vec2) {
        if self.enabled {
            self.mouse_delta += delta;
        }
    }

    pub fn update(&mut self, dt: f32, window: &Window, camera: &mut Camera) -> bool {
        if !self.enabled {
            if window.set_cursor_grab(CursorGrabMode::None).is_err() {
                log::error!("failed to unlock cursor");
            }
            window.set_cursor_visible(true);
            return false;
        }

        if window.set_cursor_grab(CursorGrabMode::Locked).is_err() {
            log::error!("failed to lock cursor");
        }
        window.set_cursor_visible(false);

        let mut updated = false;

        let up = glam::vec3(0.0, 1.0, 0.0);
        let right = camera.forward.cross(up);

        let mut position_delta = glam::Vec3::ZERO;
        if self.key_inputs[0] {
            position_delta += camera.forward * self.speed * dt;
        }
        if self.key_inputs[1] {
            position_delta -= camera.forward * self.speed * dt;
        }
        if self.key_inputs[2] {
            position_delta -= right * self.speed * dt;
        }
        if self.key_inputs[3] {
            position_delta += right * self.speed * dt;
        }
        if self.key_inputs[4] {
            position_delta -= up * self.speed * dt;
        }
        if self.key_inputs[5] {
            position_delta += up * self.speed * dt;
        }

        if position_delta.length_squared() > 0.0 {
            camera.position += position_delta;
            updated = true;
        }

        if self.mouse_delta.length_squared() > 0.0 {
            let pitch_delta = self.mouse_delta.y * self.sensitivity * dt;
            let yaw_delta = self.mouse_delta.x * self.sensitivity * dt;

            let rotation = glam::Quat::from_axis_angle(right, -pitch_delta)
                .mul_quat(glam::Quat::from_axis_angle(up, -yaw_delta));
            camera.forward = rotation.mul_vec3(camera.forward);

            self.mouse_delta = glam::Vec2::ZERO;
            updated = true;
        }

        updated
    }
}
