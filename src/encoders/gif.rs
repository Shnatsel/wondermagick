use std::io::Write;

use image::codecs::gif::GifEncoder;
use image::{DynamicImage, ExtendedColorType};

use crate::{error::MagickError, image::Image, plan::Modifiers, wm_try};
// needed because of https://github.com/image-rs/image/issues/2498
use crate::encoders::webp::to_8bit_color;

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

    let converted_image = to_8bit_color(pixels);
    let image = converted_image.as_ref().unwrap_or(&image.pixels);
    Ok(match image {
        DynamicImage::ImageRgb8(data) => {
            wm_try!(encoder.encode(data.as_raw(), width, height, ExtendedColorType::Rgb8))
        }
        DynamicImage::ImageRgba8(data) => {
            wm_try!(encoder.encode(data.as_raw(), width, height, ExtendedColorType::Rgba8))
        }
        _ => unreachable!(), // we've just converted it to RGB(A)
    })
}
