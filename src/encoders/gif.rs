use std::io::Write;

use image::codecs::gif::GifEncoder;
use image::ExtendedColorType;

use crate::{error::MagickError, image::Image, plan::Modifiers, wm_try};

pub fn encode<W: Write>(
    image: &Image,
    writer: &mut W,
    _modifiers: &Modifiers,
) -> Result<(), MagickError> {
    let mut encoder = GifEncoder::new_with_speed(writer, 10);
    // TODO: ugly because of
    // https://github.com/image-rs/image/issues/2497
    // should instead be:
    // Ok(wm_try!(image.pixels.write_with_encoder(encoder)))
    let pixels = &image.pixels;
    let width = pixels.width();
    let height = pixels.height();
    use image::DynamicImage::*;
    let converted_image = match pixels {
        ImageRgb8(_) => None,
        ImageRgba8(_) => None,
        _ => if pixels.has_alpha() {
            Some(ImageRgba8(pixels.to_rgba8()))
        } else {
            Some(ImageRgb8(pixels.to_rgb8()))
        },
    };

    let image = converted_image.as_ref().unwrap_or(&image.pixels);
    Ok(match image {
        ImageRgb8(data) => {
            wm_try!(encoder.encode(data.as_raw(), width, height, ExtendedColorType::Rgb8))
        }
        ImageRgba8(data) => {
            wm_try!(encoder.encode(data.as_raw(), width, height, ExtendedColorType::Rgba8))
        }
        _ => unreachable!(), // we've just converted it to RGB(A)
    })
}
