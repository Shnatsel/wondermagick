use image::{DynamicImage, ExtendedColorType, ImageFormat};
use std::ffi::OsString;

#[derive(Debug, Clone)]
pub struct InputProperties {
    pub filename: OsString,
    pub color_type: ExtendedColorType,
}

#[derive(Debug, Clone)]
pub struct Image {
    // TODO: ImageFormat only lists the built-in formats, which is why it's an Option.
    // We need the extended format enum with a string for plug-in formats here, but it's not public (yet).
    pub format: Option<ImageFormat>,
    pub exif: Option<Vec<u8>>,
    pub icc: Option<Vec<u8>>,
    pub pixels: DynamicImage,
    pub properties: InputProperties,
}
