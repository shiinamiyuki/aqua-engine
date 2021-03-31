use nalgebra_glm as glm;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PointLightPod {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

pub struct PointLight {
    pub position: glm::Vec3,
    pub color: glm::Vec3,
}

impl From<PointLight> for PointLightPod {
    fn from(item: PointLight) -> Self {
        Self {
            position: item.position.into(),
            color: item.color.into(),
        }
    }
}

// pub struct AreaLight {
    
// }