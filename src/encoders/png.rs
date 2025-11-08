use std::borrow::Cow;
use std::fmt::Debug;
use std::io::Write;

use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::{ColorType, DynamicImage, ImageBuffer, Pixel, Primitive};

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
    let pixels_to_write = optimize_pixel_format(&image.pixels, true);
    Ok(wm_try!(pixels_to_write.write_with_encoder(encoder)))
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

// TODO: upstream all the pixel format optimization below into `image`

fn optimize_pixel_format(image: &DynamicImage, reduce_precision: bool) -> Cow<'_, DynamicImage> {
    // TODO: palettize if the image has <256 colors
    use DynamicImage::*;
    let mut transforms = match image {
        ImageLumaA8(pixels) => find_pixel_optimizations(pixels),
        ImageRgb8(pixels) => find_pixel_optimizations(pixels),
        ImageRgba8(pixels) => find_pixel_optimizations(pixels),
        ImageLuma16(pixels) => find_pixel_optimizations(pixels),
        ImageLumaA16(pixels) => find_pixel_optimizations(pixels),
        ImageRgb16(pixels) => find_pixel_optimizations(pixels),
        ImageRgba16(pixels) => find_pixel_optimizations(pixels),
        _ => return Cow::Borrowed(image), // no-op
    };

    if !reduce_precision {
        transforms.eight_bit = false;
    }

    apply_pixel_format_optimizations(image, transforms)
}

fn apply_pixel_format_optimizations(
    image: &DynamicImage,
    transforms: PixelFormatTransforms,
) -> Cow<'_, DynamicImage> {
    let mut color_type = image.color();
    if transforms.eight_bit {
        color_type = to_8bit(color_type);
    }
    if transforms.grayscale {
        color_type = to_grayscale(color_type);
    }
    if transforms.opaque {
        color_type = to_opaque(color_type);
    }

    dynimage_to_color(image, color_type)
}

/// Converts the specified color type to its grayscale equivalent, if possible.
/// `RgbF32` and `RgbaF32` are left intact because there is no grayscale equivalent for them.
fn to_grayscale(color: ColorType) -> ColorType {
    match color {
        ColorType::Rgb8 => ColorType::L8,
        ColorType::Rgba8 => ColorType::La8,
        ColorType::Rgb16 => ColorType::L16,
        ColorType::Rgba16 => ColorType::La16,
        other => other,
    }
}

fn to_8bit(color: ColorType) -> ColorType {
    match color {
        ColorType::L16 => ColorType::L8,
        ColorType::La16 => ColorType::La8,
        ColorType::Rgb16 => ColorType::Rgb8,
        ColorType::Rgba16 => ColorType::Rgba8,
        ColorType::Rgb32F => ColorType::Rgb8,
        ColorType::Rgba32F => ColorType::Rgba8,
        already_8bit => already_8bit,
    }
}

fn to_opaque(color: ColorType) -> ColorType {
    match color {
        ColorType::La8 => ColorType::L8,
        ColorType::Rgba8 => ColorType::Rgb8,
        ColorType::La16 => ColorType::L16,
        ColorType::Rgba16 => ColorType::Rgb16,
        ColorType::Rgba32F => ColorType::Rgb32F,
        already_opaque => already_opaque,
    }
}

fn dynimage_to_color(image: &DynamicImage, color: ColorType) -> Cow<'_, DynamicImage> {
    if image.color() == color {
        Cow::Borrowed(image)
    } else {
        match color {
            ColorType::L8 => Cow::Owned(DynamicImage::ImageLuma8(image.to_luma8())),
            ColorType::La8 => Cow::Owned(DynamicImage::ImageLumaA8(image.to_luma_alpha8())),
            ColorType::Rgb8 => Cow::Owned(DynamicImage::ImageRgb8(image.to_rgb8())),
            ColorType::Rgba8 => Cow::Owned(DynamicImage::ImageRgba8(image.to_rgba8())),
            ColorType::L16 => Cow::Owned(DynamicImage::ImageLuma16(image.to_luma16())),
            ColorType::La16 => Cow::Owned(DynamicImage::ImageLumaA16(image.to_luma_alpha16())),
            ColorType::Rgb16 => Cow::Owned(DynamicImage::ImageRgb16(image.to_rgb16())),
            ColorType::Rgba16 => Cow::Owned(DynamicImage::ImageRgba16(image.to_rgba16())),
            ColorType::Rgb32F => Cow::Owned(DynamicImage::ImageRgb32F(image.to_rgb32f())),
            ColorType::Rgba32F => Cow::Owned(DynamicImage::ImageRgba32F(image.to_rgba32f())),
            _ => unreachable!(),
        }
    }
}

#[inline]
fn obviously_grayscale<P: Pixel>() -> bool {
    P::CHANNEL_COUNT < 3
}

#[inline]
fn is_grayscale<P: Pixel>(pixel: P) -> bool {
    if obviously_grayscale::<P>() {
        true
    } else {
        let c = pixel.channels();
        (c[0] == c[1]) & (c[0] == c[2])
    }
}

#[inline]
fn obviously_opaque<P: Pixel>() -> bool {
    !P::HAS_ALPHA
}

#[inline]
fn is_opaque<P: Pixel>(pixel: P) -> bool {
    if obviously_opaque::<P>() {
        true
    } else {
        // This assumes that the alpha channel is always the last.
        // This holds for all DynamicImage variants but isn't safe to expose to fully generic code.
        // Unfortunately there is no "give me your alpha channel" method on Pixel.
        let alpha = *pixel.channels().last().unwrap();
        alpha == P::Subpixel::DEFAULT_MAX_VALUE
    }
}

#[inline]
fn contains_8_bit_data<S: Primitive + Debug, P: Pixel<Subpixel = S>>(pixel: P) -> bool {
    if obviously_8bit::<S, P>() {
        true
    } else if Some(S::DEFAULT_MAX_VALUE) == S::from(65535) {
        pixel
            .channels()
            .iter()
            .copied()
            .all(|channel_value| channel_value % S::from(257).unwrap() == S::from(0).unwrap())
    } else {
        false
    }
}

#[inline]
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
    #[inline]
    fn all_true() -> Self {
        Self {
            grayscale: true,
            opaque: true,
            eight_bit: true,
        }
    }

    #[inline]
    fn all_false() -> Self {
        Self {
            grayscale: false,
            opaque: false,
            eight_bit: false,
        }
    }
}

fn find_pixel_optimizations<S: Primitive + Debug, P: Pixel<Subpixel = S>, Container>(
    input: &ImageBuffer<P, Container>,
) -> PixelFormatTransforms
where
    P: Pixel + 'static,
    Container: std::ops::Deref<Target = [P::Subpixel]>,
    P::Subpixel:,
{
    // all transforms are assumed to be valid until proven invalid
    let mut result = PixelFormatTransforms::all_true();

    // Check for all properties in a single scan through memory.
    for row in input.rows() {
        for pixel in row {
            // We still check for properties that might already be found not to hold
            // to avoid branching inside this hot loop and to enable autovectorization.
            // Checks for properties that are statically known not to hold
            // (e.g. 8-bit images containing 8-bit data) are optimized out at compile time
            // because this function is generic on the pixel format.
            result.grayscale &= is_grayscale(*pixel);
            result.opaque &= is_opaque(*pixel);
            result.eight_bit &= contains_8_bit_data(*pixel);
        }
        // If we've proven all properties to be false, short-cirquit.
        // We do this once per row to enable autovectorization above.
        // Ideally you'd want this per chunk rather than per row,
        // but rows() returning an iterator over Pixels instead of a slice makes this hard.
        if result == PixelFormatTransforms::all_false() {
            return result;
        }
    }

    result
}
