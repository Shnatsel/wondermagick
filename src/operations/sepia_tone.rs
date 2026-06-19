use image::{DynamicImage, ImageBuffer, Pixel, Primitive};
use num_traits::NumCast;
use std::fmt::Debug;

use crate::{arg_parsers::SepiaThreshold, error::MagickError, image::Image, wm_err};

pub fn sepia_tone(image: &mut Image, threshold: &SepiaThreshold) -> Result<(), MagickError> {
    match &mut image.pixels {
        DynamicImage::ImageLuma8(pixels) => sepia_tone_inner(pixels, threshold),
        DynamicImage::ImageLumaA8(pixels) => sepia_tone_inner(pixels, threshold),
        DynamicImage::ImageRgb8(pixels) => sepia_tone_inner(pixels, threshold),
        DynamicImage::ImageRgba8(pixels) => sepia_tone_inner(pixels, threshold),
        DynamicImage::ImageLuma16(pixels) => sepia_tone_inner(pixels, threshold),
        DynamicImage::ImageLumaA16(pixels) => sepia_tone_inner(pixels, threshold),
        DynamicImage::ImageRgb16(pixels) => sepia_tone_inner(pixels, threshold),
        DynamicImage::ImageRgba16(pixels) => sepia_tone_inner(pixels, threshold),
        DynamicImage::ImageRgb32F(pixels) => sepia_tone_inner(pixels, threshold),
        DynamicImage::ImageRgba32F(pixels) => sepia_tone_inner(pixels, threshold),
        _ => unreachable!(),
    }
}

fn sepia_tone_inner<P, Container>(
    buffer: &mut ImageBuffer<P, Container>,
    _threshold: &SepiaThreshold,
) -> Result<(), MagickError>
where
    P: Pixel + Debug,
    P::Subpixel: Primitive + Debug,
    Container: std::ops::DerefMut<Target = [P::Subpixel]>,
{
    if P::CHANNEL_COUNT < 3 {
        return Ok(());
    }

    let max_val: f32 = NumCast::from(P::Subpixel::DEFAULT_MAX_VALUE).unwrap();

    for pixel in buffer.pixels_mut() {
        let (r, g, b): (f32, f32, f32) = {
            let c = pixel.channels();
            (
                NumCast::from(c[0]).ok_or(wm_err!("foo"))?,
                NumCast::from(c[1]).ok_or(wm_err!("bar"))?,
                NumCast::from(c[2]).ok_or(wm_err!("baz"))?,
            )
        };

        let new_r = (0.393 * r + 0.769 * g + 0.189 * b).clamp(0.0, max_val);
        let new_g = (0.349 * r + 0.686 * g + 0.168 * b).clamp(0.0, max_val);
        let new_b = (0.272 * r + 0.534 * g + 0.131 * b).clamp(0.0, max_val);

        let c = pixel.channels_mut();
        c[0] = NumCast::from(new_r).ok_or(wm_err!("foo"))?;
        c[1] = NumCast::from(new_g).ok_or(wm_err!("bar"))?;
        c[2] = NumCast::from(new_b).ok_or(wm_err!("baz"))?;
    }
    Ok(())
}
