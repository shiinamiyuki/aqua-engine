use super::{DeviceContext, Size};

pub trait ComputePass {
    type Input;
    fn record_command(
        &mut self,
        size: Size,
        ctx: &mut DeviceContext,
        input: &Self::Input,
    ) -> wgpu::CommandBuffer;
}
