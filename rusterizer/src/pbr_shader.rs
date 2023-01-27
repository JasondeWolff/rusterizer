use crate::graphics::{Shader, ShaderIn};
use crate::resources::Material;
use crate::glam::Vec4;

pub struct PBRShader;

impl Shader for PBRShader {
    fn shade(&self, material: &Material, inputs: &ShaderIn) -> Vec4 {
        let tex_coord = &inputs.tex_coord;

        let mut color = Vec4::default();

        if let Some(base_color_texture) = material.base_color_texture.try_as_ref() {
            color = Vec4::from((base_color_texture.get_pixel_vec3(tex_coord.x, tex_coord.y), 1.0));
        }
        //color = Vec4::from((inputs.normal, 1.0));

        color
    }
}