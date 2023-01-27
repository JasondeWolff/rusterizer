use crate::glam::*;

use crate::resources::Image;
use crate::Shared;

#[derive(Clone)]
pub struct Material {
    pub name: String,
    pub index: Option<usize>,

    pub base_color_factor: Vec4,
    pub base_color_texture: Shared<Image>,

    pub normal_scale: f32,
    pub normal_texture: Shared<Image>,

    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub metallic_roughness_texture: Shared<Image>,

    pub occlusion_strength: f32,
    pub occlusion_texture: Shared<Image>,

    pub emissive_factor: Vec3,
    pub emissive_texture: Shared<Image>,
}

impl Default for Material {
    fn default() -> Self {
        Material {
            name: String::from("default"),
            index: None,
            base_color_factor: Vec4::new(1.0, 1.0, 1.0, 1.0),
            base_color_texture: Shared::empty(),
            normal_scale: 1.0,
            normal_texture: Shared::empty(),
            metallic_factor: 0.0,
            roughness_factor: 1.0,
            metallic_roughness_texture: Shared::empty(),
            occlusion_strength: 1.0,
            occlusion_texture: Shared::empty(),
            emissive_factor: Vec3::default(),
            emissive_texture: Shared::empty(),
        }
    }
}

#[derive(Clone)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub tangent: Vec4,
    pub tex_coord: Vec2,
    pub tex_coord_1: Vec2,
    pub color: Vec4
}

impl Default for Vertex {
    fn default() -> Self {
        Vertex {
            position: Vec3::default(),
            normal: Vec3::default(),
            tangent: Vec4::default(),
            tex_coord: Vec2::default(),
            tex_coord_1: Vec2::default(),
            color: Vec4::default()
        }
    }
}

#[derive(Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,

    pub min: Vec3,
    pub max: Vec3,

    pub material_idx: usize
}

#[derive(Clone)]
pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Shared<Material>>
}