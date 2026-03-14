pub mod fetch;
pub mod image_ops;
pub mod scan;
pub mod thumbnails;

pub struct Image {
    pub width: u32,
    pub height: u32,
    pub bytes: Vec<u8>,
}

pub type Color = (u8, u8, u8, f32);
