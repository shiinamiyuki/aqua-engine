use wgpu::util::DeviceExt;

use crate::geometry::TriangleMesh;

use super::{DeviceContext, RenderContext};
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex<const N: usize>([f32; N]);

unsafe impl bytemuck::Pod for Vertex<2> {}
unsafe impl bytemuck::Zeroable for Vertex<2> {}

unsafe impl bytemuck::Pod for Vertex<3> {}
unsafe impl bytemuck::Zeroable for Vertex<3> {}

unsafe impl bytemuck::Pod for Vertex<4> {}
unsafe impl bytemuck::Zeroable for Vertex<4> {}

pub struct VertexBuffer {
    stride: usize,
    data:wgpu::Buffer,
    format:wgpu::VertexFormat,
    name:String,
    
}


pub struct Mesh {
    pub vertices: Vec<Vertex<3>>,
    pub normals: Vec<Vertex<3>>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn from_triangle_mesh(trig: &TriangleMesh) -> Mesh {
        if trig.vertices.len() != trig.normals.len() {
            panic!("invalid mesh, please duplicate/merge vertices");
        }

        let mut indices = vec![];
        for face in trig.indices.iter() {
            indices.push(face[0]);
            indices.push(face[1]);
            indices.push(face[2]);
        }
        let vertices: Vec<_> = (0..trig.vertices.len())
            .map(|i| {
                Vertex([
                    trig.vertices[i][0],
                    trig.vertices[i][1],
                    trig.vertices[i][2],
                ])
            })
            .collect();
        let normals: Vec<_> = (0..trig.vertices.len())
            .map(|i| Vertex::<3>([trig.normals[i][0], trig.normals[i][1], trig.normals[i][2]]))
            .collect();
        Mesh {
            vertices,
            normals,
            indices,
        }
    }
}

pub struct GPUMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub normal_buffer: wgpu::Buffer,
    pub num_indices: u32,
}

impl GPUMesh {
    pub fn new(ctx: &DeviceContext, mesh: &Mesh) -> GPUMesh {
        let vertex_buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&mesh.vertices[..]),
                usage: wgpu::BufferUsage::VERTEX,
            });
        let normal_buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Normal Buffer"),
                contents: bytemuck::cast_slice(&mesh.normals[..]),
                usage: wgpu::BufferUsage::VERTEX,
            });
        let index_buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&mesh.indices[..]),
                usage: wgpu::BufferUsage::INDEX,
            });
        let num_indices = mesh.indices.len() as u32;
        Self {
            vertex_buffer,
            normal_buffer,
            index_buffer,
            num_indices,
        }
    }
    pub fn render<'a, 'b>(&'a self, render_pass: &mut wgpu::RenderPass<'b>)
    where
        'a: 'b,
    {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.normal_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1); // 2.
    }
}
