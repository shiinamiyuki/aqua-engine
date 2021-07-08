use std::{fs::File, path::Path, sync::Arc};

use crate::render::Mesh;
use crate::*;

use super::{Camera, DeviceContext, GPUMesh, PointLight};
use akari::{scenegraph::node, shape::TriangleMesh, util};
use nalgebra::Point;

pub struct GPUScene {
    pub meshes: Vec<Arc<GPUMesh>>,
    pub point_lights: Vec<PointLight>,
}

struct SceneLoaderContext<'a> {
    parent_path: &'a Path,
    graph: &'a node::Scene,
    shapes: Vec<Mesh>,
    camera: Option<Camera>,
    lights: Vec<PointLight>,
    // named_bsdfs: HashMap<String, Arc<dyn Bsdf>>,
}
impl<'a> SceneLoaderContext<'a> {
    fn resolve_file<P: AsRef<Path>>(&self, path: P) -> File {
        if let Ok(file) = File::open(&path) {
            return file;
        }
        {
            let _guard = util::CurrentDirGuard::new();
            std::env::set_current_dir(self.parent_path).unwrap();
            if let Ok(file) = File::open(&path) {
                return file;
            }
        }
        panic!("cannot resolve path {}", path.as_ref().display());
    }
    pub fn load(&mut self, ctx: &DeviceContext) -> GPUScene {
        let mut meshes = vec![];
        let mut point_lights = vec![];
        for shape in self.graph.shapes.iter() {
            match shape {
                node::Shape::Mesh(path, _bsdf) => {
                    let mut file = self.resolve_file(path);

                    // let bsdf = self.load_bsdf(bsdf_node);
                    let model = {
                        let bson_data = bson::Document::from_reader(&mut file).unwrap();
                        bson::from_document::<TriangleMesh>(bson_data).unwrap()
                    };
                    meshes.push(Arc::new(GPUMesh::new(
                        ctx,
                        &Mesh::from_triangle_mesh(&model),
                    )));
                }
            }
        }
        for light in self.graph.lights.iter() {
            match light {
                node::Light::Point { pos, emission: _ } => {
                    point_lights.push(PointLight {
                        position: *pos,
                        emission: glm::vec3(1.0, 1.0, 1.0),
                    });
                }
            }
        }
        GPUScene {
            meshes,
            point_lights,
        }
    }
}
impl GPUScene {
    pub fn load_scene(path: &Path, device_ctx: &DeviceContext) -> GPUScene {
        let serialized = std::fs::read_to_string(path).unwrap();
        let canonical = std::fs::canonicalize(path).unwrap();
        let graph: node::Scene = serde_json::from_str(&serialized).unwrap();
        let mut ctx = SceneLoaderContext {
            parent_path: &canonical.parent().unwrap(),
            graph: &graph,
            shapes: vec![],
            lights: vec![],
            camera: None,
        };
        ctx.load(device_ctx)
    }
    // pub fn draw<'a, 'b>(&'a self, render_pass: &mut wgpu::RenderPass<'b>)
    // where
    //     'a: 'b,
    // {
    //     for m in &self.meshes {
    //         m.render(render_pass);
    //         // render_pass.set_vertex_buffer(0, m.vertex_buffer.slice(..));
    //         // render_pass.set_index_buffer(m.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
    //         // render_pass.draw_indexed(0..m.num_indices, 0, 0..1);
    //     }
    // }
}
