#[derive(Clone, Debug)]
pub struct Camera {
    pub position: glam::Vec3,
    pub forward: glam::Vec3,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: glam::vec3(0.0, 0.0, 1.0),
            forward: -glam::Vec3::Z,
            aspect: 1.0,
            fovy: 45.0,
            znear: 0.1,
            zfar: 1000.0,
        }
    }
}

impl Camera {
    pub fn validate(&self) -> bool {
        self.aspect > 0.0 && self.znear > 0.0 && self.zfar > 0.0
    }

    pub fn compute_projection(&self) -> glam::Mat4 {
        let fovy_radians = std::f32::consts::PI / 180.0 * self.fovy;
        glam::Mat4::perspective_rh(fovy_radians, self.aspect, self.znear, self.zfar)
    }

    pub fn compute_inverse_projection(&self) -> glam::Mat4 {
        self.compute_projection().inverse()
    }

    pub fn compute_view(&self) -> glam::Mat4 {
        glam::Mat4::look_at_rh(
            self.position,
            self.position + self.forward,
            glam::Vec3::new(0.0, 1.0, 0.0),
        )
    }

    pub fn compute_inverse_view(&self) -> glam::Mat4 {
        self.compute_view().inverse()
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuCamera {
    position: glam::Vec3,
    pad0: u32,
    inverse_projection: glam::Mat4,
    inverse_view: glam::Mat4,
}

impl From<Camera> for GpuCamera {
    fn from(camera: Camera) -> Self {
        Self {
            position: camera.position,
            pad0: 0,
            inverse_projection: camera.compute_inverse_projection(),
            inverse_view: camera.compute_inverse_view(),
        }
    }
}
