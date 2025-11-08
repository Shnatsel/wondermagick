use std::io::Write;

use image::codecs::jpeg::JpegEncoder;

use crate::encoders::common::{optimize_pixel_format, write_icc_and_exif};
use crate::{error::MagickError, image::Image, plan::Modifiers, wm_try};

pub fn encode<W: Write>(
    image: &Image,
    writer: &mut W,
    modifiers: &Modifiers,
) -> Result<(), MagickError> {
    // imagemagick estimates the quality of the input JPEG somehow according to
    // https://www.imagemagick.org/script/command-line-options.php#quality
    // but we don't do that yet
    let quality = match modifiers.quality {
        Some(q) => convert_quality(q),
        None => 92, // imagemagick default
    };
    let mut encoder = JpegEncoder::new_with_quality(writer, quality);
    write_icc_and_exif(&mut encoder, image);
    let pixels_to_write = optimize_pixel_format(&image.pixels);
    wm_try!(pixels_to_write.write_with_encoder(encoder));
    Ok(())
}

fn convert_quality(input: f64) -> u8 {
    // `as` is a saturating cast that rounds down.
    // This appears to match the behavior of imagemagick
    let quality = input as i64;
    // For JPEG you can set negative quality and it maps to 100.
    // Qualities above it are also set to 100.
    match quality {
        0..=100 => quality as u8,
        _ => 100,
    }
}
