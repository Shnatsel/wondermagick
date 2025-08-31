use image::{DynamicImage, ImageFormat};

#[derive(Debug, Clone)]
pub struct Image {
    pub format: ImageFormat,
    pub exif: Option<Vec<u8>>,
    pub icc: Option<Vec<u8>>,
    pub pixels: DynamicImage,
}
