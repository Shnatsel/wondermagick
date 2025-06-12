use std::ffi::OsStr;

use crate::{error::MagickError, wm_try};
use image::DynamicImage;

#[derive(Debug, Clone)]
pub struct Image {
    pub exif: Option<Vec<u8>>,
    pub icc: Option<Vec<u8>>,
    pub pixels: DynamicImage,
}

impl Image {
    pub fn save(&self, output_file: &OsStr) -> Result<(), MagickError> {
        Ok(wm_try!(self.pixels.save(output_file)))
        // TODO: Exif, ICC once they are implemented in `image`
        // TODO: handle -strip flag here. It's not an operation because `-strip -auto-orient` works.
    }
}
