use crate::glam::IVec2;
use crate::glam::Vec3;

#[derive(Clone)]
pub struct Image {
    pub data: Vec<u8>,
    pub dimensions: IVec2,
    pub channel_count: i32
}

impl Image {
    pub fn get_pixel_vec3(&self, x: f32, y: f32) -> Vec3 {
        let x = ((x * self.dimensions.x as f32) as usize).clamp(0, self.dimensions.x as usize - 1);
        let y = ((y * self.dimensions.y as f32) as usize).clamp(0, self.dimensions.y as usize - 1);

        if self.channel_count == 3 {
            let data: &Vec<(u8, u8, u8)> = unsafe { std::mem::transmute(&self.data) };
            let pixel = &data[y * (self.dimensions.x as usize) + x];

            return Vec3::new(pixel.0 as f32 / 255.99, pixel.1 as f32 / 255.99, pixel.2 as f32 / 255.99);
        }
        
        Vec3::default()
    }
}