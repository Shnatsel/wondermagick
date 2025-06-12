use std::ffi::OsStr;

use image::{DynamicImage, ImageDecoder, ImageFormat, ImageReader};

use crate::{error::MagickError, image::Image, wm_try};

/// If the format has not been explicitly specified, guesses the format based on file contents.
pub fn decode(file: &OsStr, format: Option<ImageFormat>) -> Result<Image, MagickError> {
    let mut reader = wm_try!(ImageReader::open(file));
    match format {
        Some(format) => reader.set_format(format),
        None => reader = wm_try!(reader.with_guessed_format()),
    }
    let mut decoder = wm_try!(reader.into_decoder());
    let exif = decoder.exif_metadata().unwrap_or(None);
    let icc = decoder.icc_profile().unwrap_or(None);
    let pixels = wm_try!(DynamicImage::from_decoder(decoder));
    Ok(Image { exif, icc, pixels })
}
