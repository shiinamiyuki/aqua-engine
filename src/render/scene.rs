use std::sync::Arc;

use super::{GPUMesh, PointLight};

pub struct Scene {}

pub struct GPUScene {
    pub meshes: Vec<Arc<GPUMesh>>,
    pub point_lights: Vec<PointLight>,
}

impl GPUScene {
    pub fn draw<'a, 'b>(&'a self, render_pass: &mut wgpu::RenderPass<'b>)
    where
        'a: 'b,
    {
        for m in &self.meshes {
            render_pass.set_vertex_buffer(0, m.vertex_buffer.slice(..));
            render_pass.set_index_buffer(m.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..m.num_indices, 0, 0..1);
        }
    }
}
