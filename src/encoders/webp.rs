use std::io::Write;

use crate::encoders::common::to_8bit_rgb_maybe_a;
use crate::{error::MagickError, image::Image, plan::Modifiers, wm_err, wm_try};
use webp::{Encoder, WebPMemory};

pub fn encode<W: Write>(
    image: &Image,
    writer: &mut W,
    modifiers: &Modifiers,
) -> Result<(), MagickError> {
    // Convert the image to Rgb(a)8, because those are the only formats the encoder supports
    let pixels = to_8bit_rgb_maybe_a(&image.pixels);
    let encoder: Encoder = Encoder::from_image(pixels.as_ref()).unwrap();
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
