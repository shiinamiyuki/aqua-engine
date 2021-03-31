pub struct RenderPipeline {
    pub pipeline: wgpu::RenderPipeline,
}

impl RenderPipeline {
    pub fn new(pipeline: wgpu::RenderPipeline) -> RenderPipeline {
        Self { pipeline }
    }
}
