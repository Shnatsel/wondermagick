use std::io::Write;

use crate::{
    encoders::common::is_opaque, error::MagickError, image::Image, plan::Modifiers, wm_err, wm_try,
};
use image::DynamicImage;
use webp::{Encoder, WebPMemory};

pub fn encode<W: Write>(
    image: &Image,
    writer: &mut W,
    modifiers: &Modifiers,
) -> Result<(), MagickError> {
    // Convert the image to Rgb(a)8, because those are the only formats the encoder supports
    let converted_pixels = to_8bit_color(&image.pixels);
    let pixels = converted_pixels.as_ref().unwrap_or(&image.pixels);

    let encoder: Encoder = Encoder::from_image(pixels).unwrap();
    // imagemagick signals that the image should be lossless with quality=100
    let lossless = modifiers.quality == Some(100.0);
    // default quality is not documented, was determined experimentally
    let quality = modifiers.quality.unwrap_or(75.0) as f32;

    // Encode the image with the specified quality
    let webp: WebPMemory = encoder
        .encode_simple(lossless, quality)
        .map_err(|e| wm_err!("WebP encoding failed: {e:?}"))?;
    // TODO: `webp` crate doesn't support setting the ICC profile:
    // https://github.com/jaredforth/webp/issues/41
    Ok(wm_try!(writer.write_all(&webp)))
}

/// Converts the input image to Rgb8 or Rgba8, depending on the presence of an alpha channel.
/// Returns `None` if the image doesn't need conversion.
pub(crate) fn to_8bit_color(pixels: &DynamicImage) -> Option<DynamicImage> {
    use image::DynamicImage::*;
    match pixels {
        ImageRgb8(_) => None,
        ImageRgba8(_) => None,
        _ => {
            if pixels.color().has_alpha() {
                if is_opaque(pixels) {
                    Some(ImageRgb8(pixels.to_rgb8()))
                } else {
                    Some(ImageRgba8(pixels.to_rgba8()))
                }
            } else {
                Some(ImageRgb8(pixels.to_rgb8()))
            }
        }
    }
}
