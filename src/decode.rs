use std::ffi::OsStr;

use image::{DynamicImage, ImageDecoder, ImageFormat, ImageReader};

use crate::{error::MagickError, wm_try};

/// If the format has not been explicitly specified, guesses the format based on file contents.
pub fn decode(file: &OsStr, format: Option<ImageFormat>) -> Result<DynamicImage, MagickError> {
    let mut reader = wm_try!(ImageReader::open(file));
    match format {
        Some(format) => reader.set_format(format),
        None => reader = wm_try!(reader.with_guessed_format()),
    }
    let mut decoder = wm_try!(reader.into_decoder());
    let orientation = wm_try!(decoder.orientation());
    let mut image = wm_try!(DynamicImage::from_decoder(decoder));
    // TODO: apply orientation only if -auto-orient is passed
    image.apply_orientation(orientation);
    Ok(image)
}
