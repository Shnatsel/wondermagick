use std::ffi::OsStr;

use image::{DynamicImage, ImageDecoder, ImageFormat, ImageReader, ImageResult};

/// If the format has not been explicitly specified, guesses the format based on file contents.
pub fn decode(file: &OsStr, format: Option<ImageFormat>) -> ImageResult<DynamicImage> {
    let mut reader = ImageReader::open(file)?;
    match format {
        Some(format) => reader.set_format(format),
        None => reader = reader.with_guessed_format()?,
    }
    let mut decoder = reader.into_decoder()?;
    let exif = decoder.exif_metadata();
    // TODO: apply EXIF rotation
    DynamicImage::from_decoder(decoder)
}
