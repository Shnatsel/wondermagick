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
    pub xmp: Option<Vec<u8>>,
    pub icc: Option<Vec<u8>>,
    pub pixels: DynamicImage,
    pub properties: InputProperties,
}

impl Image {
    /// Refreshes the reportable color type after an operation changes the image color model.
    ///
    /// Decoding may preserve an extended input color type that is not representable in
    /// `DynamicImage`, and `identify` should report that original type until wondermagick changes
    /// the colorspace itself. Color-changing operations such as grayscale, monochrome, and combine
    /// should call this after replacing `pixels`.
    pub fn set_color_type_from_pixels(&mut self) {
        self.properties.color_type = self.pixels.color().into();
    }
}
