use nalgebra_glm as glm;

use super::BufferData;

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PointLightData {
    pub position: [f32; 3],
    pub _pad0: f32,
    pub emission: [f32; 3],
}

pub struct PointLight {
    pub position: glm::Vec3,
    pub emission: glm::Vec3,
}

impl BufferData for PointLightData {
    type Native = PointLight;
    fn new(value: &Self::Native) -> Self {
        Self {
            position: value.position.into(),
            emission: value.emission.into(),
            _pad0:0.0,
        }
    }
}
