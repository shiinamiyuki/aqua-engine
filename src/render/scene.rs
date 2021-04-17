use std::sync::Arc;

use super::{GPUMesh, PointLight};

pub struct Scene {

}

pub struct GPUScene {
    pub meshes: Vec<Arc<GPUMesh>>,
    pub point_lights: Vec<PointLight>,
}