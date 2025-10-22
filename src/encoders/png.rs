use std::borrow::Cow;
use std::io::Write;
use std::result;

use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::{DynamicImage, Pixel, Primitive};

use crate::encoders::common::write_icc_and_exif;
use crate::plan::Modifiers;
use crate::wm_err;
use crate::{error::MagickError, image::Image, wm_try};

pub fn encode<W: Write>(
    image: &Image,
    writer: &mut W,
    modifiers: &Modifiers,
) -> Result<(), MagickError> {
    let (compression, filter) = quality_to_compression_parameters(modifiers.quality)?;
    let mut encoder = PngEncoder::new_with_quality(writer, compression, filter);
    write_icc_and_exif(&mut encoder, image);
    Ok(wm_try!(image.pixels.write_with_encoder(encoder)))
}

// for documentation on conversion of quality to encoding parameters see
// https://www.imagemagick.org/script/command-line-options.php#quality
fn quality_to_compression_parameters(
    quality: Option<f64>,
) -> Result<(CompressionType, FilterType), MagickError> {
    if let Some(quality) = quality {
        if quality.is_sign_negative() {
            return Err(wm_err!("PNG quality cannot be negative"));
        }
        let quality = quality as u64;

        let compression = match quality / 10 {
            n @ 0..=9 => CompressionType::Level(n as u8),
            10.. => CompressionType::Level(9), // in imagemagick large values are treated as 9
        };
        let filter = match quality % 10 {
            0 => FilterType::NoFilter,
            1 => FilterType::Sub,
            2 => FilterType::Up,
            3 => FilterType::Avg,
            4 => FilterType::Paeth,
            // 7 is documented as MNG-only, in practice maps to 5 or 6?
            5..=7 => FilterType::Adaptive,
            // filters 8 and 9 override compression level selection
            8 => return Ok((CompressionType::Fast, FilterType::Adaptive)),
            // imagemagick uses filter=None here, but our Fast mode needs filtering
            // to deliver reasonable compression, so use the fastest filter instead
            9 => return Ok((CompressionType::Fast, FilterType::Up)),
            10.. => unreachable!(),
        };

        if filter == FilterType::NoFilter && compression == CompressionType::Fast {
            // CompressionType::Fast needs filtering for a reasonable compression ratio.
            // When using it, use the fastest filter instead of no filter at all.
            Ok((CompressionType::Fast, FilterType::Up))
        } else {
            Ok((compression, filter))
        }
    } else {
        // default is 75 as per https://legacy.imagemagick.org/script/command-line-options.php#quality
        Ok((CompressionType::Level(7), FilterType::Adaptive))
    }
}

fn optimize_pixel_format(pixels: &DynamicImage) -> Cow<DynamicImage> {
    // TODO: palettize if the image has <256 colors
    use DynamicImage::*;
    match pixels {
        ImageRgb8(pixels) => match pixel_opt_transforms(pixels) {},
        ImageRgba8(pixels) => todo!(),
        ImageLuma16(pixels) => todo!(),
        ImageLumaA16(pixels) => todo!(),
        ImageRgb16(pixels) => todo!(),
        ImageRgba16(pixels) => todo!(),
        _ => Cow::Borrowed(pixels), // no-op
    }
}

fn obviously_grayscale<P: Pixel>() -> bool {
    P::CHANNEL_COUNT < 3
}

fn is_grayscale<P: Pixel>(input: &[P]) -> bool {
    if obviously_grayscale::<P>() {
        true
    } else {
        input.iter().copied().all(|pixel| {
            let c = pixel.channels();
            c[0] == c[1] && c[0] == c[2]
        })
    }
}

fn obviously_opaque<P: Pixel>() -> bool {
    !P::HAS_ALPHA
}

fn is_opaque<P: Pixel>(input: &[P]) -> bool {
    if obviously_opaque::<P>() {
        true
    } else {
        match input.first() {
            Some(first_pixel) => {
                // TODO: assumes that the alpha channel is always last.
                // This holds for all DynamicImage variants but isn't safe to expose to fully generic code.
                // Unfortunately there is no "give me your alpha channel" method on Pixel.
                let first_alpha = *first_pixel.channels().last().unwrap();
                input
                    .iter()
                    .copied()
                    .all(|pixel| *pixel.channels().last().unwrap() == first_alpha)
            }
            None => true,
        }
    }
}

fn contains_8_bit_data<S: Primitive, P: Pixel<Subpixel = S>>(input: &[P]) -> bool {
    if obviously_8bit::<S, P>() {
        true
    } else if Some(S::DEFAULT_MAX_VALUE) == S::from(65535) {
        input.iter().copied().all(|pixel| {
            pixel
                .channels()
                .iter()
                .copied()
                .all(|channel_value| channel_value % S::from(256).unwrap() == S::from(0).unwrap())
        })
    } else {
        false
    }
}

fn obviously_8bit<S: Primitive, P: Pixel<Subpixel = S>>() -> bool {
    Some(S::DEFAULT_MAX_VALUE) == S::from(255)
}

#[derive(Copy, Clone, PartialEq, Eq)]
struct PixelFormatTransforms {
    grayscale: bool,
    opaque: bool,
    eight_bit: bool,
}

impl PixelFormatTransforms {
    fn all_true() -> Self {
        Self {
            grayscale: true,
            opaque: true,
            eight_bit: true,
        }
    }

    fn all_false() -> Self {
        Self {
            grayscale: false,
            opaque: false,
            eight_bit: false,
        }
    }
}

fn pixel_opt_transforms<S: Primitive, P: Pixel<Subpixel = S>>(
    input: &[P],
) -> PixelFormatTransforms {
    // all transforms are assumed to be valid until proven invalid
    let mut result = PixelFormatTransforms::all_true();

    // Skip searching for possible transforms at runtime if they're obviously invalid at compile time.
    // E.g. don't check 8-bit images to confirm that all values are within the 8-bit range.
    if obviously_grayscale::<P>() {
        result.grayscale = false
    }
    if obviously_opaque::<P>() {
        result.opaque = false
    }
    if obviously_8bit::<S, P>() {
        result.eight_bit = false
    }

    // Check for all properties in a single scan through memory.
    // We also check in chunks rather than pixel by pixel to allow for autovectorization.
    // These two tricks improve performance significantly.
    for chunk in input.chunks_exact(16) {
        // we can stop checking for each property as soon as we find that it doesn't hold
        if result.grayscale {
            result.grayscale &= is_grayscale(chunk)
        }
        if result.opaque {
            result.opaque &= is_opaque(chunk)
        }
        if result.eight_bit {
            result.eight_bit &= contains_8_bit_data(chunk)
        }
        // If we've proven all properties to be false, short-cirquit
        if result == PixelFormatTransforms::all_false() {
            return result;
        }
    }
    // check the remainder after the chunked iteration
    let chunk = input.chunks_exact(16).remainder();
    if result.grayscale {
        result.grayscale &= is_grayscale(chunk)
    }
    if result.opaque {
        result.opaque &= is_opaque(chunk)
    }
    if result.eight_bit {
        result.eight_bit &= contains_8_bit_data(chunk)
    }

    result
}
