use super::{Buffer, BufferData};
use glm::Vec3;
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
pub struct ViewProjection(pub glm::Mat4, pub glm::Mat4);

impl Default for ViewProjection {
    fn default() -> Self {
        Self(glm::identity(), glm::identity())
    }
}
impl Default for UniformViewProjection {
    fn default() -> Self {
        Self {
            view: glm::identity::<f32, 4>().into(),
            proj: glm::identity::<f32, 4>().into(),
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
pub fn opengl_to_wgpu_matrix()-> glm::Mat4 { 
    glm::Mat4::from_column_slice(&[
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 0.5, 0.0,
        0.0, 0.0, 0.5, 1.0
    ])
}

// impl Camera {
//     pub fn build_view_projection_matrix(&self) -> ViewProjection {
//
//     }
// }
// pub trait Camera {
//     fn build_view_projection_matrix(&self) -> ViewProjection;
//     fn pos(&self) -> Vec3;
//     fn dir(&self) -> Vec3;
// }
#[derive(Clone)]
pub struct OribitalCamera {
    pub perspective: Perspective,
    pub center: glm::Vec3,
    pub radius: f32,
    pub phi: f32,
    pub theta: f32,
}
#[derive(Clone)]
pub struct LookAtCamera {
    pub perspective: Perspective,
    pub eye: glm::Vec3,
    pub center: glm::Vec3,
    pub up: glm::Vec3,
}

impl LookAtCamera {
    pub fn build_view_projection_matrix(&self) -> ViewProjection {
        let view = glm::look_at(&self.eye, &self.center, &self.up);
        let proj = glm::perspective(
            self.perspective.aspect,
            self.perspective.fovy,
            self.perspective.znear,
            self.perspective.zfar,
        );
        ViewProjection(view, opengl_to_wgpu_matrix() * proj)
    }
    pub fn pos(&self) -> Vec3 {
        self.eye
    }
    pub fn dir(&self) -> Vec3 {
        glm::normalize(&(self.center - self.eye))
    }
}

impl OribitalCamera {
    pub fn build_view_projection_matrix(&self) -> ViewProjection {
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
        ViewProjection(view, opengl_to_wgpu_matrix() * proj)
    }
    pub fn pos(&self) -> Vec3 {
        let dir = glm::vec3(
            self.phi.sin() * self.theta.sin(),
            self.theta.cos(),
            self.phi.cos() * self.theta.sin(),
        );
        self.center + self.radius * dir
    }
    pub fn dir(&self) -> Vec3 {
        let dir = glm::vec3(
            self.phi.sin() * self.theta.sin(),
            self.theta.cos(),
            self.phi.cos() * self.theta.sin(),
        );
        -dir
    }
}

#[derive(Clone)]
pub enum Camera {
    Orbital(OribitalCamera),
    LookAt(LookAtCamera),
}

impl Camera {
    pub fn build_view_projection_matrix(&self) -> ViewProjection {
        match &self {
            Camera::Orbital(orbital) => orbital.build_view_projection_matrix(),
            Camera::LookAt(look_at) => look_at.build_view_projection_matrix(),
        }
    }
    pub fn pos(&self) -> Vec3 {
        match &self {
            Camera::Orbital(orbital) => orbital.pos(),
            Camera::LookAt(look_at) => look_at.pos(),
        }
    }
    pub fn dir(&self) -> Vec3 {
        match &self {
            Camera::Orbital(orbital) => orbital.dir(),
            Camera::LookAt(look_at) => look_at.dir(),
        }
    }
}
