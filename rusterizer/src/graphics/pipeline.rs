use crate::glam::*;
use crate::window::FrameBuffer;
use crate::resources::Vertex;
use crate::resources::Material;
use crate::graphics::{Shader, ShaderIn};

fn from_vec3_rgb(rgb: &Vec3) -> u32 {
    from_u8_rgb((rgb.x * 255.99) as u8, (rgb.y * 255.99) as u8, (rgb.z * 255.99) as u8)
}

fn from_u8_rgb(r: u8, g: u8, b: u8) -> u32 {
    let (r, g, b) = (r as u32, g as u32, b as u32);
    (r << 16) | (g << 8) | b
}

pub struct Pipeline {
    model_matrix: Mat4,
    view_matrix: Mat4,
    proj_matrix: Mat4,

    depth_buffer: FrameBuffer
}

impl Pipeline {
    pub fn new() -> Self {
        Pipeline {
            model_matrix: Mat4::IDENTITY,
            view_matrix: Mat4::IDENTITY,
            proj_matrix: Mat4::IDENTITY,
            depth_buffer: FrameBuffer::new(0, 0)
        }
    }

    pub fn set_model_matrix(&mut self, model_matrix: Mat4) {
        self.model_matrix = model_matrix;
    }

    pub fn set_view_matrix(&mut self, view_matrix: Mat4) {
        self.view_matrix = view_matrix;
    }

    pub fn set_proj_matrix(&mut self, proj_matrix: Mat4) {
        self.proj_matrix = proj_matrix;
    }

    pub fn clear_depth(&mut self) {
        self.depth_buffer.clear(u32::MAX);
    }

    pub fn draw_vertices(&mut self, shader: &dyn Shader, material: &Material, frame_buffer: &mut FrameBuffer, vertices: &Vec<Vertex>) {
        self.adapt_depth_buffer(&frame_buffer);

        let mvp =  self.proj_matrix * self.view_matrix * self.model_matrix;

        let triangle_count = vertices.len() / 3;
        for i in 0..triangle_count {
            let v0 = &vertices[(i * 3 + 0) as usize];
            let v1 = &vertices[(i * 3 + 1) as usize];
            let v2 = &vertices[(i * 3 + 2) as usize];

            let a = Self::project(&v0.position, &mvp);
            let b = Self::project(&v1.position, &mvp);
            let c = Self::project(&v2.position, &mvp);

            Self::draw_triangle(shader, material, frame_buffer, &mut self.depth_buffer, &a, &b, &c, v0, v1, v2, &self.model_matrix);
        }
    }

    pub fn draw_vertices_indexed(&mut self, shader: &dyn Shader, material: &Material, frame_buffer: &mut FrameBuffer, vertices: &Vec<Vertex>, indices: &Vec<u32>) {
        self.adapt_depth_buffer(&frame_buffer);

        let mvp =  self.proj_matrix * self.view_matrix * self.model_matrix;

        let triangle_count = indices.len() / 3;
        for i in 0..triangle_count {
            let v0 = &vertices[indices[(i * 3 + 0) as usize] as usize];
            let v1 = &vertices[indices[(i * 3 + 1) as usize] as usize];
            let v2 = &vertices[indices[(i * 3 + 2) as usize] as usize];

            let a = Self::project(&v0.position, &mvp);
            let b = Self::project(&v1.position, &mvp);
            let c = Self::project(&v2.position, &mvp);

            Self::draw_triangle(shader, material, frame_buffer, &mut self.depth_buffer, &a, &b, &c, v0, v1, v2, &self.model_matrix);
        }
    }

    fn clip_to_screen_space(clip_space: Vec2, screen_size: Vec2) -> Vec2 {
        (clip_space * -0.5 + 0.5) * screen_size
    }

    fn adapt_depth_buffer(&mut self, frame_buffer: &FrameBuffer) {
        if self.depth_buffer.width() != frame_buffer.width() || self.depth_buffer.height() != frame_buffer.height() {
            self.depth_buffer = FrameBuffer::new(frame_buffer.width(), frame_buffer.height());
        }
    }

    fn draw_triangle(shader: &dyn Shader, material: &Material, frame_buffer: &mut FrameBuffer, depth_buffer: &mut FrameBuffer, a: &(Vec3, f32), b: &(Vec3, f32), c: &(Vec3, f32), v0: &Vertex, v1: &Vertex, v2: &Vertex, model_matrix: &Mat4) {
        let rec0 = a.1;
        let rec1 = b.1;
        let rec2 = c.1;
        
        let a = &a.0;
        let b = &b.0;
        let c = &c.0;

        let z0 = a.z;
        let z1 = b.z;
        let z2 = c.z;

        let a = Self::clip_to_screen_space(Vec2::new(a.x, a.y), Vec2::new(frame_buffer.width() as f32, frame_buffer.height() as f32));
        let b = Self::clip_to_screen_space(Vec2::new(b.x, b.y), Vec2::new(frame_buffer.width() as f32, frame_buffer.height() as f32));
        let c = Self::clip_to_screen_space(Vec2::new(c.x, c.y), Vec2::new(frame_buffer.width() as f32, frame_buffer.height() as f32));

        let min = a.min(b.min(c)).max(Vec2::new(0.0, 0.0));
        let max = a.max(b.max(c)).min(Vec2::new(frame_buffer.width() as f32, frame_buffer.height() as f32));

        for x in (min.x as usize)..=(max.x as usize) {
            for y in (min.y as usize)..=(max.y as usize) {
                let p = Vec2::new(x as f32, y as f32) + 0.5;

                let a0 = Self::edge_function(&b, &c, &p);
                let a1 = Self::edge_function(&c, &a, &p);
                let a2 = Self::edge_function(&a, &b, &p);

                let mut overlaps = true;

                let edge0 = c - b;
                let edge1 = a - c;
                let edge2 = b - a;
                overlaps &= if a0 == 0.0 { (edge0.y == 0.0 && edge0.x > 0.0) ||  edge0.y > 0.0 } else { a0 > 0.0 };
                overlaps &= if a1 == 0.0 { (edge1.y == 0.0 && edge1.x > 0.0) ||  edge1.y > 0.0 }  else { a1 > 0.0 };
                overlaps &= if a1 == 0.0 { (edge2.y == 0.0 && edge2.x > 0.0) ||  edge2.y > 0.0 }  else { a2 > 0.0 };

                if overlaps {
                    let area_rep = 1.0 / Self::edge_function(&a, &b, &c);
                    let bary = Vec3::new(a0, a1, a2) * area_rep;
                    
                    let z = z0 * bary.x + z1 * bary.y + z2 * bary.z;
                    let d = depth_buffer.get_pixel_f32(x, y);
                    if z < d {
                        depth_buffer.set_pixel_f32(x, y, z);

                        // model space -> world space
                        let mv0 = *model_matrix * Vec4::from((v0.position * rec0, 1.0));
                        let mv1 = *model_matrix * Vec4::from((v1.position * rec1, 1.0));
                        let mv2 = *model_matrix * Vec4::from((v2.position * rec2, 1.0));

                        let correction = 1.0 / (bary.x * rec0 + bary.y * rec1 + bary.z * rec2);

                        let position = mv0 * bary.x + mv1 * bary.y + mv2 * bary.z;
                        let normal = v0.normal * rec0 * bary.x + v1.normal * rec1 * bary.y + v2.normal * rec2 * bary.z;
                        let tex_coord = v0.tex_coord * rec0 * bary.x + v1.tex_coord * rec1 * bary.y + v2.tex_coord * rec2 * bary.z;

                        let shader_in = ShaderIn {
                            position: Vec3::new(position.x, position.y, position.z) * correction,
                            normal: normal * correction,
                            tex_coord: tex_coord * correction
                        };

                        let color = shader.shade(material, &shader_in);
                        let color = Vec3::new(color.x, color.y, color.z);
                        frame_buffer.set_pixel(x, y, from_vec3_rgb(&color));
                    }
                }
            }
        }
    }

    fn project(p: &Vec3, mvp: &Mat4) -> (Vec3, f32) {
        let proj_pos = *mvp * Vec4::from((*p, 1.0));
        let rec = 1.0 / proj_pos.w;
        let rec_pos = proj_pos * rec;
        (Vec3::new(rec_pos.x, rec_pos.y, rec_pos.z), rec)
    }

    fn edge_function(a: &Vec2, c: &Vec2, b: &Vec2) -> f32 {
        (c.x - a.x) * (b.y - a.y) - (c.y - a.y) * (b.x - a.x)
    }
}