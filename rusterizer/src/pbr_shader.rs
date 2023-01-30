use crate::graphics::{Shader, ShaderIn};
use crate::resources::Material;
use crate::glam::*;

use std::f32::consts::PI;

pub struct PBRShader {
    pub view_position: Vec3,
    pub sample_bilinear: bool
}

impl Default for PBRShader {
    fn default() -> Self {
        PBRShader {
            view_position: Vec3::default(),
            sample_bilinear: true
        }
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a * (1.0 - t) + (b * t)
}

fn lerp_v3(a: Vec3, b: Vec3, t: f32) -> Vec3 {
    a * (1.0 - t) + (b * t)
}

// ----------------------------------------------------------------------------
fn distribution_ggx(n: &Vec3, h: Vec3, roughness: f32) -> f32
{
    let a = roughness*roughness;
    let a2 = a*a;
    let n_dot_h = n.dot(h).max(0.0);
    let n_dot_h2 = n_dot_h*n_dot_h;

    let nom   = a2;
    let denom = n_dot_h2 * (a2 - 1.0) + 1.0;
    let denom = PI * denom * denom;

    nom / denom
}
// ----------------------------------------------------------------------------
fn geometry_schlick_ggx(n_dot_v: f32, roughness:  f32) -> f32
{
    let r = roughness + 1.0;
    let k = (r * r) / 8.0;

    let nom   = n_dot_v;
    let denom = n_dot_v * (1.0 - k) + k;

    return nom / denom;
}
// ----------------------------------------------------------------------------
fn geometry_smith(n: &Vec3, v: Vec3, l: Vec3, roughness: f32) -> f32
{
    let n_dot_v = n.dot(v).max(0.0);
    let n_dot_l = n.dot(l).max(0.0);
    let ggx2 = geometry_schlick_ggx(n_dot_v, roughness);
    let ggx1 = geometry_schlick_ggx(n_dot_l, roughness);

    return ggx1 * ggx2;
}
// ----------------------------------------------------------------------------
fn fresnel_schlick(cos_theta: f32, f0: Vec3) -> Vec3
{
    //return F0 + (1.0 - F0) * pow(clamp(1.0 - cosTheta, 0.0, 1.0), 5.0);
    f0 + (1.0 - f0) * (1.0 - cos_theta).clamp(0.0, 1.0).powf(5.0)
}

impl Shader for PBRShader {
    fn shade(&self, material: &Material, inputs: &ShaderIn) -> Vec4 {
        let tex_coord = &inputs.tex_coord;

        let mut base_color = material.base_color_factor.xyz();
        if let Some(base_color_texture) = material.base_color_texture.try_as_ref() {
            base_color *= base_color_texture.sample_pixel(tex_coord.x, tex_coord.y, self.sample_bilinear).xyz();
        }

        let n = &inputs.normal;

        let mut metallic = material.metallic_factor;
        let mut roughness = material.roughness_factor;
        if let Some(metallic_roughness_texture) = material.metallic_roughness_texture.try_as_ref() {
            let metallic_roughness = metallic_roughness_texture.sample_pixel(tex_coord.x, tex_coord.y, self.sample_bilinear).yz();
            metallic *= metallic_roughness.y;
            roughness *= metallic_roughness.x;
        }

        let mut occlusion = 1.0;
        if let Some(occlusion_texture) = material.occlusion_texture.try_as_ref() {
            occlusion = lerp(occlusion_texture.sample_pixel(tex_coord.x, tex_coord.y, self.sample_bilinear).x, 1.0, 1.0 - material.occlusion_strength);
        }

        let mut emission = Vec3::default();
        if let Some(emissive_texture) = material.emissive_texture.try_as_ref() {
            emission = emissive_texture.sample_pixel(tex_coord.x, tex_coord.y, self.sample_bilinear).xyz() * material.emissive_factor;
        }

        let v = (self.view_position - inputs.position).normalize();
        let ao = 0.1;

        let mut f0 = Vec3::splat(0.04);
        f0 = lerp_v3(f0, base_color, metallic);

        let lo;
        {
            let l = (-Vec3::new(0.1, -1.0, 0.0)).normalize();
            let h = (v + l).normalize();
            let radiance = Vec3::splat(1.1);

            let ndf = distribution_ggx(n, h, roughness);   
            let g = geometry_smith(n, v, l, roughness);      
            let f = fresnel_schlick(h.dot(v).clamp(0.0, 1.0), f0);

            let numerator = ndf * g * f;
            let denominator = 4.0 * n.dot(v).max(0.0) * n.dot(l).max(0.0) + 0.0001;
            let specular = numerator / denominator;

            let ks = f;
            let mut kd = Vec3::splat(1.0) - ks;
            kd *= 1.0 - metallic;

            let n_dot_l = n.dot(l).max(0.0);
            lo = (kd * base_color / PI + specular) * radiance * n_dot_l;
        }

        let ambient = Vec3::splat(0.03) * base_color * ao;

        let mut color = (ambient + lo) * occlusion + emission;
        color = color / (color + 1.0);
        color = color.powf(1.0 / 2.2);

        Vec4::from((color, 1.0))
    }
}