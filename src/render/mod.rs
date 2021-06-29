pub mod buffer;
pub mod camera;
pub mod context;
pub mod gbuffer;
pub mod light;
pub mod mesh;
pub mod pass;
pub mod pipeline;
pub mod shader;
pub mod ssrt;
pub mod texture;
// pub mod svo;
pub mod scene;

pub use buffer::*;
pub use camera::*;
pub use context::*;
pub use gbuffer::*;
pub use light::*;
pub use mesh::*;
pub use pass::*;
pub use pipeline::*;
pub use scene::*;
pub use shader::*;
pub use ssrt::*;
pub use texture::*;
// pub use svo::*;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}
impl Size {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

pub fn fovy_to_fovx(fovy: f32, aspect: f32) -> f32 {
    // fieldOfViewX = 2 * atan(tan(fieldOfViewY * 0.5) * aspect)
    ((fovy * 0.5).tan() * aspect).atan()
}
pub fn fovx_to_fovy(fovx: f32, aspect: f32) -> f32 {
    // fieldOfViewX = 2 * atan(tan(fieldOfViewY * 0.5) * aspect)
    // fieldOfViewX*0.5 = atan(tan(fieldOfViewY * 0.5) * aspect)
    // tan(fieldOfViewX*0.5) = tan(fieldOfViewY * 0.5) * aspect
    // fieldOfViewY = 2 * atan(tan(fieldOfViewX*0.5)/aspect)
    ((fovx * 0.5).tan() / aspect).atan()
}
