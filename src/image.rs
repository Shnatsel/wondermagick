use image::DynamicImage;

#[derive(Debug, Clone)]
pub struct Image {
    pub exif: Option<Vec<u8>>,
    pub icc: Option<Vec<u8>>,
    pub pixels: DynamicImage,
}
