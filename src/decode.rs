use std::ffi::OsStr;

use image::{DynamicImage, ImageFormat, ImageReader, ImageResult};

/// If the format has not been explicitly specified, guesses the format based on file contents.
pub fn decode(file: &OsStr, format: Option<ImageFormat>) -> ImageResult<DynamicImage> {
    let mut decoder = ImageReader::open(file)?;
    match format {
        Some(format) => decoder.set_format(format),
        None => decoder = decoder.with_guessed_format()?,
    }
    decoder.decode()
}