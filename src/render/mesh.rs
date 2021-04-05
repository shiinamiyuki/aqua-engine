use crate::render::state::State;
use wgpu::util::DeviceExt;
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}
impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float3,
                },
            ],
        }
    }
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn from_triangle_mesh(trig: &crate::TriangleMesh) -> Mesh {
        if trig.vertices.len() != trig.normals.len() {
            panic!("invalid mesh, please duplicate/merge vertices");
        }
        let mut vertices = vec![];
        let mut indices = vec![];
        for face in trig.indices.iter() {
            indices.push(face[0]);
            indices.push(face[1]);
            indices.push(face[2]);
        }
        for i in 0..trig.vertices.len() {
            let vert = Vertex {
                position: [
                    trig.vertices[i][0],
                    trig.vertices[i][1],
                    trig.vertices[i][2],
                ],
                normal: [trig.normals[i][0], trig.normals[i][1], trig.normals[i][2]],
            };
            vertices.push(vert);
        }
        Mesh { vertices, indices }
    }
}

pub struct GPUMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
}

impl GPUMesh {
    pub fn new(state: &mut State, mesh: &Mesh) -> GPUMesh {
        let vertex_buffer = state
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&mesh.vertices[..]),
                usage: wgpu::BufferUsage::VERTEX,
            });
        let index_buffer = state
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&mesh.indices[..]),
                usage: wgpu::BufferUsage::INDEX,
            });
        let num_indices = mesh.indices.len() as u32;
        Self {
            vertex_buffer,
            index_buffer,
            num_indices,
        }
    }
    pub fn render<'a, 'b>(&'a self, render_pass: &mut wgpu::RenderPass<'b>)
    where
        'a: 'b,
    {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1); // 2.
    }
}
