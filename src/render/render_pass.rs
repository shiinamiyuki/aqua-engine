use std::sync::Arc;

use super::{Buffer, BufferData, FrameContext, PointLight, Texture, UniformViewProjection, Vertex};
use super::{Camera, GPUMesh, Size};
use super::{PointLightData, RenderContext};
use crate::glm;
use env_logger::fmt::Color;

pub struct ColorAttachment<'a> {
    pub view: &'a wgpu::TextureView,
}
impl <'a> ColorAttachment<'a> {
    pub fn get_descriptor(&self) -> wgpu::RenderPassColorAttachmentDescriptor {
        wgpu::RenderPassColorAttachmentDescriptor {
            attachment: &self.view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                }),
                store: true,
            },
        }
    }
}
// pub struct RenderInput<'a, 'b> {
//     pub attachements: Vec<ColorAttachment<'b>>,
//     pub meshes: &'a [GPUMesh],
// }
pub trait RenderPass {
    type Input;
    fn record_command(
        &mut self,
        size: Size,
        ctx: &mut RenderContext,
        frame_ctx: &mut FrameContext,
        camera: &dyn Camera,
        input: &Self::Input,
    ) -> wgpu::CommandBuffer;
}
