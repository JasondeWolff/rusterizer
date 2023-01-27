use crate::glam::IVec2;

#[derive(Clone)]
pub struct Image {
    pub data: Vec<u8>,
    pub dimensions: IVec2,
    pub channel_count: i32
}