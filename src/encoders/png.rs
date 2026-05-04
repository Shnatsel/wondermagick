use std::borrow::Cow;
use std::io::Write;

use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::DynamicImage;

use crate::encoders::common::{optimize_pixel_format_and_precision, write_metadata};
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
    write_metadata(&mut encoder, image);
    // we need to coerce f32 to u16 ourselves here before it goes through pixel format optimization
    let pixels = coerce_pixel_format(&image.pixels);
    let pixels_to_write = optimize_pixel_format_and_precision(&pixels);
    wm_try!(pixels_to_write.write_with_encoder(encoder));
    Ok(())
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

/// Coerces f32 pixel data to u16, returns an unchanged DynamicImage otherwise
fn coerce_pixel_format(image: &DynamicImage) -> Cow<'_, DynamicImage> {
    use DynamicImage::*;
    match image {
        ImageRgb32F(_) => Cow::Owned(ImageRgb16(image.to_rgb16())),
        ImageRgba32F(_) => Cow::Owned(ImageRgba16(image.to_rgba16())),
        _ => Cow::Borrowed(image), // no-op
    }
}
