use super::{Buffer, BufferData};
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
pub struct ViewProjection(glm::Mat4, glm::Mat4);

impl Default for ViewProjection {
    fn default() -> Self {
        Self(glm::identity(), glm::identity())
    }
}
impl Default for UniformViewProjection {
    fn default() -> Self {
        Self {
            view: glm::identity::<f32, na::U4>().into(),
            proj: glm::identity::<f32, na::U4>().into(),
            // model: glm::identity::<f32, na::U4>().into(),
        }
    }
}
impl BufferData for UniformViewProjection {
    type Native = ViewProjection;
    fn new(vp: &ViewProjection) -> Self {
        let ViewProjection(view, proj) = vp;
        Self {
            view: (*view).into(),
            proj: (*proj).into(),
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Perspective {
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

// pub struct Camera {
//     pub eye: glm::Vec3,
//     pub center: glm::Vec3,
//     pub up: glm::Vec3,
//     pub aspect: f32,
//     pub fovy: f32,
//     pub znear: f32,
//     pub zfar: f32,
// }
#[rustfmt::skip]
    pub const OPENGL_TO_WGPU_MATRIX: [f32;16] =  [
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 0.5, 0.0,
        0.0, 0.0, 0.5, 1.0
    ];

// impl Camera {
//     pub fn build_view_projection_matrix(&self) -> ViewProjection {
//
//     }
// }
pub trait Camera {
    fn build_view_projection_matrix(&self) -> ViewProjection;
}

pub struct OribitalCamera {
    pub perspective: Perspective,
    pub center: glm::Vec3,
    pub radius: f32,
    pub phi: f32,
    pub theta: f32,
}

pub struct LookAtCamera {
    pub perspective: Perspective,
    pub eye: glm::Vec3,
    pub center: glm::Vec3,
    pub up: glm::Vec3,
}

impl Camera for LookAtCamera {
    fn build_view_projection_matrix(&self) -> ViewProjection {
        let view = glm::look_at(&self.eye, &self.center, &self.up);
        let proj = glm::perspective(
            self.perspective.aspect,
            self.perspective.fovy,
            self.perspective.znear,
            self.perspective.zfar,
        );
        ViewProjection(
            view,
            glm::Mat4::from_row_slice(&OPENGL_TO_WGPU_MATRIX[..]) * proj,
        )
    }
}

impl Camera for OribitalCamera {
    fn build_view_projection_matrix(&self) -> ViewProjection {
        let dir = glm::vec3(
            self.phi.sin() * self.theta.sin(),
            self.theta.cos(),
            self.phi.cos() * self.theta.sin(),
        );
        let eye = self.center + self.radius * dir;
        let view = glm::look_at(&eye, &self.center, &glm::vec3(0.0, 1.0, 0.0));
        let proj = glm::perspective(
            self.perspective.aspect,
            self.perspective.fovy,
            self.perspective.znear,
            self.perspective.zfar,
        );
        ViewProjection(
            view,
            glm::Mat4::from_row_slice(&OPENGL_TO_WGPU_MATRIX[..]) * proj,
        )
    }
}
