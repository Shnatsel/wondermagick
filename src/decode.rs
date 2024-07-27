use std::ffi::OsStr;

use image::{DynamicImage, ImageDecoder, ImageFormat, ImageReader};

use crate::{error::MagickError, exif::rotate_by_exif, wm_try};

/// If the format has not been explicitly specified, guesses the format based on file contents.
pub fn decode(file: &OsStr, format: Option<ImageFormat>) -> Result<DynamicImage, MagickError> {
    let mut reader = wm_try!(ImageReader::open(file));
    match format {
        Some(format) => reader.set_format(format),
        None => reader = wm_try!(reader.with_guessed_format()),
    }
    let mut decoder = wm_try!(reader.into_decoder());
    let exif = decoder.exif_metadata();
    let mut image = wm_try!(DynamicImage::from_decoder(decoder));
    if let Ok(Some(exif)) = exif {
        // we ignore errors here because a malformed exif orientation
        // should not cause the entire decoding to fail
        let _ = rotate_by_exif(&mut image, exif);
    }
    Ok(image)
}
