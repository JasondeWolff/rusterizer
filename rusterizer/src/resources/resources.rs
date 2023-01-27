extern crate gltf;
extern crate stb_image;
extern crate bitmask_enum;
use bitmask_enum::bitmask;

use std::fs;
use std::ffi::CString;
use std::path::Path;

use crate::glam::*;
use crate::Shared;

pub mod resource_manager;
pub use resource_manager::*;

pub mod model;
pub use model::*;

pub mod image;
pub use image::*;

#[bitmask(u8)]
pub enum ImageImportSettings {
    FlipVertical
}

pub struct Resources {
    model_manager: ResourceManager<Model>,
    text_manager: ResourceManager<String>,
    image_manager: ResourceManager<Image>,

    pub kill_time: f32
}

impl Resources {
    pub fn init() -> Box<Resources> {
        Box::new(Resources {
            model_manager: ResourceManager::new(5.0),
            text_manager: ResourceManager::new(5.0),
            image_manager: ResourceManager::new(5.0),
            kill_time: 5.0
        })
    }

    pub fn update(&mut self) {
        self.model_manager.update();
        self.text_manager.update();
        self.image_manager.update();
    }

    fn process_tex(&mut self, texture: &gltf::Texture, base_path: &String) -> Shared<Image> {
        let img = texture.source();
        let img = match img.source() {
            gltf::image::Source::Uri { uri, .. } => {
                let base_path = Path::new(base_path);
                let path = base_path.parent().unwrap_or_else(|| Path::new("./")).join(uri);
                self.get_image(path.into_os_string().into_string().unwrap(), Some(ImageImportSettings::FlipVertical))
            }
            _ => panic!("Failed to process tex. (Only uri support)")
        };
        img
    }

    fn process_node(&mut self, node: &gltf::Node, buffers: &Vec<gltf::buffer::Data>, _images: &Vec<gltf::image::Data>, base_path: &String, meshes: &mut Vec<Mesh>, materials: &mut Vec<Material>) {
        let (translation, rotation, scale) = node.transform().decomposed();
        let _translation = Vec3::new(translation[0], translation[1], translation[2]);
        let _rotation = Quat::from_xyzw(rotation[3], rotation[0], rotation[1], rotation[2]); // Correct order?!?!?!?
        let _scale = Vec3::new(scale[0], scale[1], scale[2]);

        match node.mesh() {
            Some(mesh) => {
                for primitive in mesh.primitives() {
                    if primitive.mode() == gltf::mesh::Mode::Triangles {
                        let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

                        let bounds = primitive.bounding_box();
                        let min = Vec3::from(bounds.min);
                        let max = Vec3::from(bounds.max);

                        let positions = {
                            let iter = reader
                                .read_positions()
                                .expect("Failed to process mesh node. (Vertices must have positions)");

                            iter.map(|arr| -> Vec3 { Vec3::from(arr) }).collect::<Vec<_>>()
                        };

                        let mut vertices: Vec<Vertex> = positions
                            .into_iter()
                            .map(|position| {
                                Vertex {
                                    position: Vec3::from(position),
                                    ..Vertex::default()
                                }
                        }).collect();

                        let indices = reader
                            .read_indices()
                            .map(|read_indices| {
                                read_indices.into_u32().collect::<Vec<_>>()
                            }).expect("Failed to process mesh node. (Indices are required)");

                        if let Some(normals) = reader.read_normals() {
                            for (i, normal) in normals.enumerate() {
                                vertices[i].normal = Vec3::from(normal);
                            }
                        }

                        let mut tex_coord_channel = 0;
                        while let Some(tex_coords) = reader.read_tex_coords(tex_coord_channel) {
                            for (i, tex_coord) in tex_coords.into_f32().enumerate() {
                                match tex_coord_channel {
                                    0 => vertices[i].tex_coord = Vec2::from(tex_coord),
                                    1 => vertices[i].tex_coord_1 = Vec2::from(tex_coord),
                                    _ => {}
                                }
                            }

                            tex_coord_channel += 1;
                        }

                        if let Some(tangents) = reader.read_tangents() {
                            for (i, tangent) in tangents.enumerate() {
                                vertices[i].tangent = Vec4::from(tangent);
                            }
                        } else {
                            // Source: 2001. http://www.terathon.com/code/tangent.html
                            let mut tan1 = vec![Vec3::default(); vertices.len()];
                            let mut tan2 = vec![Vec3::default(); vertices.len()];

                            for i in (0..indices.len()).step_by(3) {
                                let i1 = indices[i + 0] as usize;
                                let i2 = indices[i + 1] as usize;
                                let i3 = indices[i + 2] as usize;
                            
                                let v1 = vertices[i1].position;
                                let v2 = vertices[i2].position;
                                let v3 = vertices[i3].position;
                            
                                let w1 = vertices[i1].tex_coord;
                                let w2 = vertices[i2].tex_coord;
                                let w3 = vertices[i3].tex_coord;
                            
                                let x1 = v2.x - v1.x;
                                let x2 = v3.x - v1.x;
                                let y1 = v2.y - v1.y;
                                let y2 = v3.y - v1.y;
                                let z1 = v2.z - v1.z;
                                let z2 = v3.z - v1.z;

                                let s1 = w2.x - w1.x;
                                let s2 = w3.x - w1.x;
                                let t1 = w2.y - w1.y;
                                let t2 = w3.y - w1.y;

                                let r = 1.0 / (s1 * t2 - s2 * t1);

                                let sdir = Vec3::new(
                                    (t2 * x1 - t1 * x2) * r,
                                    (t2 * y1 - t1 * y2) * r,
                                    (t2 * z1 - t1 * z2) * r
                                );

                                let tdir = Vec3::new(
                                    (s1 * x2 - s2 * x1) * r,
                                    (s1 * y2 - s2 * y1) * r,
                                    (s1 * z2 - s2 * z1) * r
                                );
                            
                                tan1[i1] += sdir;
                                tan1[i2] += sdir;
                                tan1[i3] += sdir;
                            
                                tan2[i1] += tdir;
                                tan2[i2] += tdir;
                                tan2[i3] += tdir;
                            }
                        
                            for i in 0..vertices.len() {
                                let n = vertices[i].normal;
                                let t = tan1[i];
                            
                                let xyz = (t - (n * n.dot(t))).normalize();
                            
                                let w;
                                if n.cross(t).dot(tan2[i]) < 0.0 {
                                    w = -1.0;
                                } else {
                                    w = 1.0;
                                }

                                vertices[i].tangent = Vec4::new(xyz.x, xyz.y, xyz.z, w);
                            }
                        }

                        if let Some(colors) = reader.read_colors(0) {
                            let colors = colors.into_rgba_f32();
                            for (i, color) in colors.enumerate() {
                                vertices[i].color = Vec4::from(color);
                            }
                        }
                        
                        let prim_material = primitive.material();
                        let pbr = prim_material.pbr_metallic_roughness();
                        let material_idx = primitive.material().index().unwrap_or(0);

                        let material = &mut materials[material_idx];
                        if material.index == None {
                            material.index = Some(material_idx);
                            material.name = prim_material.name().map(|s| s.into()).unwrap_or(String::from("Unnamed"));
                            material.base_color_factor = Vec4::from(pbr.base_color_factor());
                            material.metallic_factor = pbr.metallic_factor();
                            material.roughness_factor = pbr.roughness_factor();
                            material.emissive_factor = Vec3::from(prim_material.emissive_factor());

                            if let Some(color_tex) = pbr.base_color_texture() {
                                material.base_color_texture = self.process_tex(&color_tex.texture(), base_path);
                            }

                            if let Some(normal_tex) = prim_material.normal_texture() {
                                material.normal_texture = self.process_tex(&normal_tex.texture(), base_path);
                                material.normal_scale = normal_tex.scale();
                            }

                            if let Some(mr_tex) = pbr.metallic_roughness_texture() {
                                material.metallic_roughness_texture = self.process_tex(&mr_tex.texture(), base_path);
                            }

                            if let Some(occlusion_tex) = prim_material.occlusion_texture() {
                                material.occlusion_texture = self.process_tex(&occlusion_tex.texture(), base_path);
                                material.occlusion_strength = occlusion_tex.strength();
                            }

                            if let Some(emissive_tex) = prim_material.emissive_texture() {
                                material.emissive_texture = self.process_tex(&emissive_tex.texture(), base_path);
                            }
                        }

                        meshes.push(Mesh {
                            vertices: vertices,
                            indices: indices,
                            min: min,
                            max: max,
                            material_idx: material_idx
                        });
                    } else {
                        panic!("Failed to process mesh node. (Trying to parse a non-triangle)");
                    }
                }
            },
            None => {}
        };
    }

    pub fn get_model(&mut self, asset_path: String) -> Shared<Model> {
        match self.model_manager.get(&asset_path) {
            Some(resource) => resource,
            None => {
                let (document, buffers, images) = gltf::import(asset_path.clone()).expect("Failed to get model.");

                let mut meshes = Vec::new();
                let mut materials = vec![Material::default(); document.materials().len()];
                if materials.len() == 0 {
                    materials.push(Material::default());
                }
                
                if document.nodes().len() > 0 {
                    self.process_node(document.nodes().next().as_ref().unwrap(), &buffers, &images, &asset_path, &mut meshes, &mut materials);
                }

                let resource = Shared::new(Model {
                    meshes: meshes,
                    materials: materials.into_iter().map(|m| Shared::new(m)).collect()
                });

                self.model_manager.insert(resource.clone(), asset_path);
                resource
            }
        }
    }

    pub fn get_text(&mut self, asset_path: String) -> Shared<String> {
        match self.text_manager.get(&asset_path) {
            Some(resource) => resource,
            None => {
                let contents = fs::read_to_string(asset_path.clone()).expect("Failed to read text file.");
                let resource = Shared::new(contents);

                self.text_manager.insert(resource.clone(), asset_path);
                resource
            }
        }
    }

    pub fn get_image(&mut self, asset_path: String, import_settings: Option<ImageImportSettings>) -> Shared<Image> {
        match self.image_manager.get(&asset_path) {
            Some(resource) => resource,
            None => {
                let c_asset_path = CString::new(asset_path.as_bytes()).unwrap();

                unsafe {
                    if let Some(import_settings) = import_settings {
                        if !import_settings.contains(ImageImportSettings::FlipVertical) {
                            stb_image::stb_image::bindgen::stbi_set_flip_vertically_on_load(1);
                        }
                    }
            
                    let mut width = 0;
                    let mut height = 0;
                    let mut channels = 0;
                    let data = stb_image::stb_image::bindgen::stbi_load(
                c_asset_path.as_ptr(),
                        &mut width,
                        &mut height,
                        &mut channels,
                        0,
                    );
                    assert!(!data.is_null(), "Failed to read image.");
                    let data: Vec<u8> = std::slice::from_raw_parts(data, (width * height * channels) as usize).to_vec();

                    let resource = Shared::new(Image {
                        data: data,
                        dimensions: IVec2::new(width, height),
                        channel_count: channels
                    });

                    self.image_manager.insert(resource.clone(), asset_path);
                    resource
                }
            }
        }
    }
}