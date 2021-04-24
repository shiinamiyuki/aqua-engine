use nalgebra as na;
use nalgebra_glm as glm;
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct UniformViewProjection {
    view: [[f32; 4]; 4],
    proj: [[f32; 4]; 4],
    // model: [[f32; 4]; 4],
}
impl UniformViewProjection {
    pub fn new() -> Self {
        use nalgebra as na;
        use nalgebra_glm as glm;
        Self {
            view: glm::identity::<f32, na::U4>().into(),
            proj: glm::identity::<f32, na::U4>().into(),
            // model: glm::identity::<f32, na::U4>().into(),
        }
    }
    pub fn update_from_camera(&mut self, camera: &crate::Camera) {
        let (view, proj) = camera.build_view_projection_matrix();
        self.view = view.into();
        self.proj = proj.into();
    }
}

pub struct Camera {
    pub eye: glm::Vec3,
    pub center: glm::Vec3, // euler angle
    pub up: glm::Vec3,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}
#[rustfmt::skip]
    pub const OPENGL_TO_WGPU_MATRIX: [f32;16] =  [
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 0.5, 0.0,
        0.0, 0.0, 0.5, 1.0
    ];

impl Camera {
    fn build_view_projection_matrix(&self) -> (glm::Mat4, glm::Mat4) {
        let view = glm::look_at(&self.eye, &self.center, &self.up);
        let proj = glm::perspective(self.aspect, self.fovy, self.znear, self.zfar);
        (
            view,
            glm::Mat4::from_row_slice(&OPENGL_TO_WGPU_MATRIX[..]) * proj,
        )
    }
}
