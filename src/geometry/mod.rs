pub mod mesh;
pub mod svo;

use std::ops::{Div, Sub};

pub use mesh::*;
use na::Vector2;
pub use svo::*;

use crate::glm;
use crate::na;

#[derive(Debug, Clone, Copy)]
pub struct Bound2<T: na::Scalar> {
    pub min: glm::TVec2<T>,
    pub max: glm::TVec2<T>,
}

#[derive(Debug, Clone, Copy)]
pub struct Bound3<T: na::Scalar> {
    pub min: glm::TVec3<T>,
    pub max: glm::TVec3<T>,
}

impl<T> Bound3<T>
where
    T: glm::Number,
{
    pub fn size(&self) -> glm::TVec3<T> {
        self.max - self.min
    }
}
impl<T> Bound3<T>
where
    T: glm::Number + na::ClosedDiv,
{
    pub fn offset(&self, p: &glm::TVec3<T>) -> glm::TVec3<T> {
        (p - self.min).component_div(&self.size())
    }
}
