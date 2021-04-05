use super::{Buffer, BufferData};
use nalgebra as na;
use nalgebra_glm as glm;
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Default, Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct UniformViewProjection {
    view: [[f32; 4]; 4],
    proj: [[f32; 4]; 4],
    // model: [[f32; 4]; 4],
}
pub struct ViewProjection(glm::Mat4, glm::Mat4);

impl Default for ViewProjection{
    fn default() -> Self {
        Self(glm::identity(), glm::identity())
    }
}
impl UniformViewProjection {
    pub fn new() -> Self {
        Self {
            view: glm::identity::<f32, na::U4>().into(),
            proj: glm::identity::<f32, na::U4>().into(),
            // model: glm::identity::<f32, na::U4>().into(),
        }
    }
}
impl BufferData for UniformViewProjection {
    type Native = ViewProjection;
    fn update(&mut self, vp: &ViewProjection) {
        let ViewProjection(view, proj) = vp;
        self.view = (*view).into();
        self.proj = (*proj).into();
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
    fn build_view_projection_matrix(&self) -> ViewProjection {
        let view = glm::look_at(&self.eye, &self.center, &self.up);
        let proj = glm::perspective(self.aspect, self.fovy, self.znear, self.zfar);
        ViewProjection(
            view,
            glm::Mat4::from_row_slice(&OPENGL_TO_WGPU_MATRIX[..]) * proj,
        )
    }
}
