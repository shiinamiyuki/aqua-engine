use std::sync::Arc;
pub mod gbuffer_pass;
pub mod post_process;
pub mod shadow;
pub mod shadow_map;
pub mod ssgi;

pub use gbuffer_pass::*;
pub use post_process::*;
pub use shadow::*;
pub use shadow_map::*;
pub use ssgi::*;

use super::{DeviceContext, FrameContext, RenderContext};

pub trait RenderPass {
    type Descriptor;
    type Params;
    type Node;
    fn create_pass(desc: &Self::Descriptor) -> Self;
    fn record_command(
        &mut self,
        params: &Self::Params,
        frame_ctx: &FrameContext,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Self::Node;
}

pub trait ComputePass {
    type Descriptor;
    type Params;
    type Node;
    fn create_pass(desc: &Self::Descriptor) -> Self;
    fn record_command(
        &mut self,
        params: &Self::Params,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Self::Node;
}
