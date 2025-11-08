use image::{ColorType, DynamicImage, ImageBuffer, Pixel, Primitive};
use std::borrow::Cow;
use std::fmt::Debug;

/// Losslessly optimizes the pixel format for the image.
///
/// If the entire image is opaque, the alpha channel will be removed.
/// If the entire image is grayscale, it will be converted to grayscale pixel format.
/// If the image is in 16-bit precision but fits into 8 bits, it will be converted to 8-bit format.
pub(crate) fn optimize_pixel_format_and_precision(image: &DynamicImage) -> Cow<'_, DynamicImage> {
    optimize_pixel_format_inner(image, true)
}

/// Losslessly optimizes the pixel format for the image.
///
/// If the entire image is opaque, the alpha channel will be removed.
/// If the entire image is grayscale, it will be converted to grayscale pixel format.
///
/// Does **not** reduce 16-bit images to 8-bit, even if it's lossless;
/// use [optimize_pixel_format_and_precision] for that.
///
/// The behavior of this function over [optimize_pixel_format_and_precision] is useful
/// when the image will be converted to 8 bits anyway (e.g. BMP),
/// or when imagemagick does not optimize format and we mimick that behavior (e.g. TIFF)
pub(crate) fn optimize_pixel_format(image: &DynamicImage) -> Cow<'_, DynamicImage> {
    optimize_pixel_format_inner(image, false)
}

/// Converts the input image to Rgba8 or Rgb8, depending on whether the image is fully opaque.
///
/// This not only checks the pixel format but also scans the image to determine if there are any transparent pixels in it.
pub(crate) fn to_8bit_rgb_maybe_a(pixels: &DynamicImage) -> Cow<'_, DynamicImage> {
    use image::DynamicImage::*;
    match pixels {
        ImageRgb8(_) | ImageRgba8(_) => Cow::Borrowed(pixels),
        _ => {
            if pixels.color().has_alpha() {
                if is_opaque(pixels) {
                    Cow::Owned(ImageRgb8(pixels.to_rgb8()))
                } else {
                    Cow::Owned(ImageRgba8(pixels.to_rgba8()))
                }
            } else {
                Cow::Owned(ImageRgb8(pixels.to_rgb8()))
            }
        }
    }
}

fn is_opaque(image: &DynamicImage) -> bool {
    match image {
        DynamicImage::ImageLuma8(pixels) => is_opaque_inner(pixels),
        DynamicImage::ImageLumaA8(pixels) => is_opaque_inner(pixels),
        DynamicImage::ImageRgb8(pixels) => is_opaque_inner(pixels),
        DynamicImage::ImageRgba8(pixels) => is_opaque_inner(pixels),
        DynamicImage::ImageLuma16(pixels) => is_opaque_inner(pixels),
        DynamicImage::ImageLumaA16(pixels) => is_opaque_inner(pixels),
        DynamicImage::ImageRgb16(pixels) => is_opaque_inner(pixels),
        DynamicImage::ImageRgba16(pixels) => is_opaque_inner(pixels),
        DynamicImage::ImageRgb32F(pixels) => is_opaque_inner(pixels),
        DynamicImage::ImageRgba32F(pixels) => is_opaque_inner(pixels),
        _ => unreachable!(),
    }
}

fn is_opaque_inner<S, P, Container>(input: &ImageBuffer<P, Container>) -> bool
where
    S: Primitive + Debug,
    P: Pixel<Subpixel = S>,
    Container: std::ops::Deref<Target = [P::Subpixel]>,
{
    if !P::HAS_ALPHA {
        true
    } else {
        let mut result = true; // opaque until proven otherwise
        for row in input.rows() {
            for pixel in row {
                result &= can_remove_alpha(*pixel);
            }
            if result == false {
                return result;
            }
        }
        result
    }
}

/// Losslessly optimizes the pixel format for the image.
///
/// If the entire image is opaque, the alpha channel will be removed.
/// If the entire image is grayscale, it will be converted to grayscale pixel format.
/// If the image is in 16-bit precision but fits into 8 bits, it will be converted to 8-bit format,
/// but only when `reduce_precision` is set to `true`.
#[inline(always)] // for constant propagation of `reduce_precision`
fn optimize_pixel_format_inner(
    image: &DynamicImage,
    reduce_precision: bool,
) -> Cow<'_, DynamicImage> {
    let rp = reduce_precision;
    use DynamicImage::*;
    let mut transforms = match image {
        ImageLumaA8(pixels) => find_pixel_optimizations(pixels, rp),
        ImageRgb8(pixels) => find_pixel_optimizations(pixels, rp),
        ImageRgba8(pixels) => find_pixel_optimizations(pixels, rp),
        ImageLuma16(pixels) => find_pixel_optimizations(pixels, rp),
        ImageLumaA16(pixels) => find_pixel_optimizations(pixels, rp),
        ImageRgb16(pixels) => find_pixel_optimizations(pixels, rp),
        ImageRgba16(pixels) => find_pixel_optimizations(pixels, rp),
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
    if transforms.strip_alpha {
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
fn can_convert_to_grayscale<P: Pixel>(pixel: P) -> bool {
    if P::CHANNEL_COUNT < 3 {
        false // input already in grayscale pixel format
    } else {
        let c = pixel.channels();
        (c[0] == c[1]) & (c[0] == c[2])
    }
}

#[inline]
fn can_remove_alpha<P: Pixel>(pixel: P) -> bool {
    if !P::HAS_ALPHA {
        false // input doesn't have alpha
    } else {
        // This assumes that the alpha channel is always the last.
        // This holds for all DynamicImage variants but isn't safe to expose to fully generic code.
        // Unfortunately there is no "give me your alpha channel" method on Pixel.
        let alpha = *pixel.channels().last().unwrap();
        alpha == P::Subpixel::DEFAULT_MAX_VALUE
    }
}

#[inline]
fn can_convert_to_8bit<S: Primitive + Debug, P: Pixel<Subpixel = S>>(pixel: P) -> bool {
    if Some(S::DEFAULT_MAX_VALUE) == S::from(255) {
        false // already 8-bit
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

#[derive(Copy, Clone, PartialEq, Eq)]
struct PixelFormatTransforms {
    grayscale: bool,
    strip_alpha: bool,
    eight_bit: bool,
}

impl PixelFormatTransforms {
    #[inline]
    fn all_true() -> Self {
        Self {
            grayscale: true,
            strip_alpha: true,
            eight_bit: true,
        }
    }

    #[inline]
    fn all_false() -> Self {
        Self {
            grayscale: false,
            strip_alpha: false,
            eight_bit: false,
        }
    }
}

fn find_pixel_optimizations<S, P, Container>(
    input: &ImageBuffer<P, Container>,
    reduce_precision: bool,
) -> PixelFormatTransforms
where
    S: Primitive + Debug,
    P: Pixel<Subpixel = S>,
    Container: std::ops::Deref<Target = [P::Subpixel]>,
{
    // all transforms are assumed to be valid until proven invalid
    let mut result = PixelFormatTransforms::all_true();

    if !reduce_precision {
        result.eight_bit = false;
    }

    // Check for all properties in a single scan through memory.
    for row in input.rows() {
        for pixel in row {
            // We still check for properties that might already be found not to hold
            // to avoid branching inside this hot loop and to enable autovectorization.
            // Checks for properties that are statically known not to hold
            // (e.g. 8-bit images containing 8-bit data) are optimized out at compile time
            // because this function is generic on the pixel format.
            result.grayscale &= can_convert_to_grayscale(*pixel);
            result.strip_alpha &= can_remove_alpha(*pixel);
            // this branch should be removed by constant propagation
            if reduce_precision {
                result.eight_bit &= can_convert_to_8bit(*pixel);
            }
        }
        // If we've proven all properties to be false, short-circuit.
        // We do this once per row to enable autovectorization above.
        // Ideally you'd want this per chunk rather than per row,
        // but rows() returning an iterator over Pixels instead of a slice makes this hard.
        if result == PixelFormatTransforms::all_false() {
            return result;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use image::{GrayImage, Luma, Rgba};

    use super::*;

    #[test]
    fn rbga16_optimizes_to_luma8() {
        let mut img = GrayImage::new(100, 100);
        let start = Luma::from_slice(&[0]);
        let end = Luma::from_slice(&[255]);

        image::imageops::vertical_gradient(&mut img, start, end);

        let luma8 = DynamicImage::ImageLuma8(img);
        let rgba16 = DynamicImage::ImageRgba16(luma8.to_rgba16());
        assert!(rgba16.color() == ColorType::Rgba16);

        let optimized = optimize_pixel_format(&rgba16);
        assert!(optimized.color() == ColorType::L16);

        let optimized_with_precision = optimize_pixel_format_and_precision(&rgba16);
        assert!(optimized_with_precision.color() == ColorType::L8);
    }

    #[test]
    fn optimizer_not_overly_aggressive() {
        let mut img: ImageBuffer<Rgba<u16>, Vec<u16>> = ImageBuffer::new(100, 100);
        let start = Rgba::from_slice(&[0, 5, 15, 20]);
        let end = Rgba::from_slice(&[255, 245, 235, 225]);
        image::imageops::vertical_gradient(&mut img, start, end);

        let dynimage = DynamicImage::ImageRgba16(img);

        let optimized = optimize_pixel_format(&dynimage);
        assert!(optimized.color() == ColorType::Rgba16);

        let optimized_with_precision = optimize_pixel_format_and_precision(&dynimage);
        assert!(optimized_with_precision.color() == ColorType::Rgba16);
    }

    #[test]
    fn luma8_to_rgb_maybe_a() {
        let mut img = GrayImage::new(100, 100);
        let start = Luma::from_slice(&[0]);
        let end = Luma::from_slice(&[255]);

        image::imageops::vertical_gradient(&mut img, start, end);

        let luma8 = DynamicImage::ImageLuma8(img);
        let luma16 = DynamicImage::ImageLumaA16(luma8.to_luma_alpha16());
        assert!(luma16.color() == ColorType::La16);

        let converted = to_8bit_rgb_maybe_a(&luma16);
        assert!(converted.color() == ColorType::Rgb8);
    }
}
