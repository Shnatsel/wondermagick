use std::io::Write;

use image::codecs::jpeg::JpegEncoder;
use image::ImageEncoder;

use crate::{error::MagickError, image::Image, plan::Modifiers, wm_try};

pub fn encode<W: Write>(
    image: &Image,
    writer: &mut W,
    modifiers: &Modifiers,
) -> Result<(), MagickError> {
    // imagemagick estimates the quality of the input JPEG somehow according to
    // https://www.imagemagick.org/script/command-line-options.php#quality
    // but we don't do that yet
    let mut encoder = JpegEncoder::new_with_quality(writer, modifiers.quality.unwrap_or(92));
    if let Some(icc) = image.icc.clone() {
        let _ = encoder.set_icc_profile(icc); // ignore UnsupportedError
    };
    Ok(wm_try!(image.pixels.write_with_encoder(encoder)))
}
