use std::io::Write;

use image::codecs::gif::GifEncoder;

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
        ImageLuma8(_) => Some(ImageRgb8(pixels.to_rgb8())),
        ImageLumaA8(_) => Some(ImageRgba8(pixels.to_rgba8())),
        ImageRgb8(_) => None,
        ImageRgba8(_) => None,
        ImageLuma16(_) => Some(ImageRgb8(pixels.to_rgb8())),
        ImageLumaA16(_) => Some(ImageRgba8(pixels.to_rgba8())),
        ImageRgb16(_) => Some(ImageRgb8(pixels.to_rgb8())),
        ImageRgba16(_) => Some(ImageRgba8(pixels.to_rgba8())),
        ImageRgb32F(_) => Some(ImageRgb8(pixels.to_rgb8())),
        ImageRgba32F(_) => Some(ImageRgba8(pixels.to_rgba8())),
        _ => unimplemented!(),
    };

    let image = converted_image.as_ref().unwrap_or(&image.pixels);
    Ok(match image {
        ImageRgb8(image_buffer) => wm_try!(encoder.encode(
            image_buffer.as_raw(),
            width,
            height,
            image::ExtendedColorType::Rgb8
        )),
        ImageRgba8(image_buffer) => wm_try!(encoder.encode(
            image_buffer.as_raw(),
            width,
            height,
            image::ExtendedColorType::Rgba8
        )),
        _ => unreachable!(), // we've just converted it to RGB(A)
    })
}
