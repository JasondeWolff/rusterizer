use crate::graphics::{Shader, ShaderIn};
use crate::resources::Material;
use crate::glam::*;

pub struct PBRShader {
    pub view_position: Vec3
}

impl Default for PBRShader {
    fn default() -> Self {
        PBRShader {
            view_position: Vec3::default()
        }
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a * (1.0 - t) + (b * t)
}

fn lerp_v3(a: Vec3, b: Vec3, t: f32) -> Vec3 {
    a * (1.0 - t) + (b * t)
}

const PI: f32 = 3.14159265359;
// ----------------------------------------------------------------------------
fn DistributionGGX(N: &Vec3, H: Vec3, roughness: f32) -> f32
{
    let a = roughness*roughness;
    let a2 = a*a;
    let NdotH = N.dot(H).max(0.0);
    let NdotH2 = NdotH*NdotH;

    let nom   = a2;
    let denom = (NdotH2 * (a2 - 1.0) + 1.0);
    let denom = PI * denom * denom;

    nom / denom
}
// ----------------------------------------------------------------------------
fn GeometrySchlickGGX(NdotV: f32, roughness:  f32) -> f32
{
    let r = (roughness + 1.0);
    let k = (r*r) / 8.0;

    let nom   = NdotV;
    let denom = NdotV * (1.0 - k) + k;

    return nom / denom;
}
// ----------------------------------------------------------------------------
fn GeometrySmith(N: &Vec3, V: Vec3, L: Vec3, roughness: f32) -> f32
{
    let NdotV = N.dot(V).max(0.0);
    let NdotL = N.dot(L).max(0.0);
    let ggx2 = GeometrySchlickGGX(NdotV, roughness);
    let ggx1 = GeometrySchlickGGX(NdotL, roughness);

    return ggx1 * ggx2;
}
// ----------------------------------------------------------------------------
fn fresnelSchlick(cosTheta: f32, F0: Vec3) -> Vec3
{
    //return F0 + (1.0 - F0) * pow(clamp(1.0 - cosTheta, 0.0, 1.0), 5.0);
    F0 + (1.0 - F0) * (1.0 - cosTheta).clamp(0.0, 1.0).powf(5.0)
}

impl Shader for PBRShader {
    fn shade(&self, material: &Material, inputs: &ShaderIn) -> Vec4 {
        let tex_coord = &inputs.tex_coord;

        let mut base_color = material.base_color_factor.xyz();
        if let Some(base_color_texture) = material.base_color_texture.try_as_ref() {
            base_color *= base_color_texture.get_pixel_vec3(tex_coord.x, tex_coord.y);
        }

        let N = &inputs.normal;

        let mut metallic = material.metallic_factor;
        let mut roughness = material.roughness_factor;
        if let Some(metallic_roughness_texture) = material.metallic_roughness_texture.try_as_ref() {
            let metallic_roughness = metallic_roughness_texture.get_pixel_vec3(tex_coord.x, tex_coord.y).yz();
            metallic *= metallic_roughness.y;
            roughness *= metallic_roughness.x;
        }

        let mut occlusion = 1.0;
        if let Some(occlusion_texture) = material.occlusion_texture.try_as_ref() {
            occlusion = lerp(occlusion_texture.get_pixel_vec3(tex_coord.x, tex_coord.y).x, 1.0, 1.0 - material.occlusion_strength);
        }

        let mut emission = Vec3::default();
        if let Some(emissive_texture) = material.emissive_texture.try_as_ref() {
            emission = emissive_texture.get_pixel_vec3(tex_coord.x, tex_coord.y) * material.emissive_factor;
        }

        let V = (self.view_position - inputs.position).normalize();
        let ao = 0.1;

        let mut F0 = Vec3::splat(0.04);
        F0 = lerp_v3(F0, base_color, metallic);

        let Lo;
        {
            let L = (-Vec3::new(0.1, -1.0, 0.0)).normalize();
            let H = (V + L).normalize();
            let radiance = Vec3::splat(1.1);

            let NDF = DistributionGGX(N, H, roughness);   
            let G = GeometrySmith(N, V, L, roughness);      
            let F = fresnelSchlick(H.dot(V).clamp(0.0, 1.0), F0);

            let numerator = NDF * G * F;
            let denominator = 4.0 * N.dot(V).max(0.0) * N.dot(L).max(0.0) + 0.0001;
            let specular = numerator / denominator;

            let kS = F;
            let mut kD = Vec3::splat(1.0) - kS;
            kD *= 1.0 - metallic;

            let NdotL = N.dot(L).max(0.0);
            Lo = (kD * base_color / PI + specular) * radiance * NdotL;
        }

        let ambient = Vec3::splat(0.03) * base_color * ao;

        let mut color = (ambient + Lo) * occlusion + emission;
        color = color / (color + 1.0);
        color = color.powf(1.0 / 2.2);

        Vec4::from((color, 1.0))
    }
}