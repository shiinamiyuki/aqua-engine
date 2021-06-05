use wgpu::util::DeviceExt;

pub struct SSGIPass {
    pub pipeline: wgpu::ComputePipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
}
