use crate::glam::*;
use crate::resources::Material;

pub struct ShaderIn {
    pub position: Vec3,
    pub normal: Vec3,
    pub tex_coord: Vec2
}

pub trait Shader {
    fn shade(&self, material: &Material, inputs: &ShaderIn) -> Vec4;
}