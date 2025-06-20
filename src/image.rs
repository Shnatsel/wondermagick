use std::ffi::OsStr;

use crate::{error::MagickError, wm_try};
use image::DynamicImage;

#[derive(Debug, Clone)]
pub struct Image {
    pub exif: Option<Vec<u8>>,
    pub icc: Option<Vec<u8>>,
    pub pixels: DynamicImage,
}
