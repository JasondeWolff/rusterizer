use crate::glam::IVec2;
use crate::glam::Vec4;
use crate::glam::Vec2;

#[derive(Clone)]
pub struct Image {
    pub data: Vec<u8>,
    pub dimensions: IVec2,
    pub channel_count: i32,

    inv_dimensions: Vec2
}

impl Image {
    pub fn new(data: Vec<u8>, dimensions: IVec2, channel_count: i32) -> Self {
        Image {
            data: data,
            dimensions: dimensions,
            channel_count: channel_count,
            inv_dimensions: Vec2::new(1.0 / dimensions.x as f32, 1.0 / dimensions.y as f32)
        }
    }

    pub fn sample_pixel(&self, x: f32, y: f32, bilinear: bool) -> Vec4 {
        if bilinear {
            let tl = self.get_pixel(x - self.inv_dimensions.x, y - self.inv_dimensions.y);
            let bl = self.get_pixel(x - self.inv_dimensions.x, y + self.inv_dimensions.y);
            let br = self.get_pixel(x + self.inv_dimensions.x, y + self.inv_dimensions.y);
            let tr = self.get_pixel(x + self.inv_dimensions.x, y - self.inv_dimensions.y);
            
            let r = (br + tr) * 0.5;
            let l = (bl + tl) * 0.5;
            let t = (tr + tl) * 0.5;
            let b = (br + bl) * 0.5;
            
            let x = x * self.dimensions.x as f32;
            let y = y * self.dimensions.y as f32;
            let xd = x - ((x as i32) as f32);
            let yd = y - ((y as i32) as f32);
            
            (l.lerp(r, xd) + t.lerp(b, yd)) * 0.5
        } else {
            self.get_pixel(x, y)
        }
    }

    pub fn get_pixel(&self, x: f32, y: f32) -> Vec4 {
        let x = ((x * self.dimensions.x as f32) as usize) % (self.dimensions.x - 1) as usize;
        let y = ((y * self.dimensions.y as f32) as usize) % (self.dimensions.y - 1) as usize;

        match self.channel_count {
            4 => {
                let data: &Vec<(u8, u8, u8, u8)> = unsafe { std::mem::transmute(&self.data) };
                let pixel = &data[y * (self.dimensions.x as usize) + x];
    
                Vec4::new(pixel.0 as f32 / 255.99, pixel.1 as f32 / 255.99, pixel.2 as f32 / 255.99, pixel.3 as f32 / 255.99)
            },
            3 => {
                let data: &Vec<(u8, u8, u8)> = unsafe { std::mem::transmute(&self.data) };
                let pixel = &data[y * (self.dimensions.x as usize) + x];
    
                Vec4::new(pixel.0 as f32 / 255.99, pixel.1 as f32 / 255.99, pixel.2 as f32 / 255.99, 0.0)
            },
            2 => {
                let data: &Vec<(u8, u8)> = unsafe { std::mem::transmute(&self.data) };
                let pixel = &data[y * (self.dimensions.x as usize) + x];
    
                Vec4::new(pixel.0 as f32 / 255.99, pixel.1 as f32 / 255.99, 0.0, 0.0)
            },
            1 => {
                let data: &Vec<u8> = unsafe { std::mem::transmute(&self.data) };
                let pixel = &data[y * (self.dimensions.x as usize) + x];
    
                Vec4::new(*pixel as f32 / 255.99, 0.0, 0.0, 0.0)
            }
            _ => panic!("Failed to get pixel. (Unsupported channel count)")
        }
    }
}