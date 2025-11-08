use std::ops::{AddAssign, BitXor};

use image::{DynamicImage, ImageBuffer, Pixel};
use pic_scale_safe::ResamplingFunction;

use crate::{
    arg_parsers::{ResizeConstraint, ResizeGeometry},
    error::MagickError,
    image::Image,
    utils::fraction::Fraction,
    wm_try,
};

use crate::arg_parsers::ResizeTarget;

/// Implements `-resize` command
pub fn resize(image: &mut Image, geometry: &ResizeGeometry) -> Result<(), MagickError> {
    let (dst_width, dst_height) = compute_dimensions(&image.pixels, geometry);
    // The default algorithm is Sinc/Lancsoz3, a very high-quality one
    resize_impl(&mut image.pixels, dst_width, dst_height, Default::default())
}

/// Implements `-scale` command
pub fn scale(image: &mut Image, geometry: &ResizeGeometry) -> Result<(), MagickError> {
    let (dst_width, dst_height) = compute_dimensions(&image.pixels, geometry);
    resize_impl(
        &mut image.pixels,
        dst_width,
        dst_height,
        ResamplingFunction::Box,
    )
}

/// Implements `-sample` command
pub fn sample(image: &mut Image, geometry: &ResizeGeometry) -> Result<(), MagickError> {
    let (dst_width, dst_height) = compute_dimensions(&image.pixels, geometry);
    resize_impl(
        &mut image.pixels,
        dst_width,
        dst_height,
        ResamplingFunction::Nearest,
    )
}

/// Implements `-thumbnail` command
pub fn thumbnail(image: &mut Image, geometry: &ResizeGeometry) -> Result<(), MagickError> {
    let image = &mut image.pixels;
    let (dst_width, dst_height) = compute_dimensions(image, geometry);

    // imagemagick first downscales to 5x the target size with the cheap nearest-neighbor algorithm
    let width = image.width().min(dst_width * 5);
    let height = image.height().min(dst_height * 5);
    wm_try!(resize_impl(
        image,
        width,
        height,
        ResamplingFunction::Nearest
    ));

    // now do the actual resize to the target dimensions
    resize_impl(image, dst_width, dst_height, Default::default())
}

fn resize_impl(
    image: &mut DynamicImage,
    dst_width: u32,
    dst_height: u32,
    algorithm: ResamplingFunction,
) -> Result<(), MagickError> {
    if image.width() == dst_width && image.height() == dst_height {
        return Ok(());
    }
    let alg = algorithm; // otherwise rustfmt breaks up too-long-lines and the formatting is a mess
    let src_size = pic_scale_safe::ImageSize::new(image.width() as usize, image.height() as usize);
    let dst_size = pic_scale_safe::ImageSize::new(dst_width as usize, dst_height as usize);

    // Premultiply the image by alpha channel to avoid color bleed from fully transparent pixels.
    let mut premultiplied_by_alpha = false;
    // There is no need to premultiply for nearest-neighbor resampling because it does not perform any blending.
    // This is actually a valuable optimization because -thumbnail uses nearest-neighbor for a part of the process.
    if algorithm != ResamplingFunction::Nearest {
        premultiplied_by_alpha = premultiply_alpha_if_needed(image);
    }

    use pic_scale_safe::*;
    match image {
        DynamicImage::ImageLuma8(src) => {
            let resized = wm_try!(resize_plane8(src.as_raw(), src_size, dst_size, alg));
            *src = ImageBuffer::from_raw(dst_width, dst_height, resized).unwrap();
        }
        DynamicImage::ImageLumaA8(src) => {
            let resized = wm_try!(resize_plane8_with_alpha(
                src.as_raw(),
                src_size,
                dst_size,
                alg
            ));
            *src = ImageBuffer::from_raw(dst_width, dst_height, resized).unwrap();
        }
        DynamicImage::ImageRgb8(src) => {
            let resized = wm_try!(resize_rgb8(src.as_raw(), src_size, dst_size, alg));
            *src = ImageBuffer::from_raw(dst_width, dst_height, resized).unwrap();
        }
        DynamicImage::ImageRgba8(src) => {
            let resized = wm_try!(resize_rgba8(src.as_raw(), src_size, dst_size, alg));
            *src = ImageBuffer::from_raw(dst_width, dst_height, resized).unwrap();
        }
        DynamicImage::ImageLuma16(src) => {
            let resized = wm_try!(resize_plane16(src.as_raw(), src_size, dst_size, 16, alg));
            *src = ImageBuffer::from_raw(dst_width, dst_height, resized).unwrap();
        }
        DynamicImage::ImageLumaA16(src) => {
            let resized = wm_try!(resize_plane16_with_alpha(
                src.as_raw(),
                src_size,
                dst_size,
                16,
                alg
            ));
            *src = ImageBuffer::from_raw(dst_width, dst_height, resized).unwrap();
        }
        DynamicImage::ImageRgb16(src) => {
            let resized = wm_try!(resize_rgb16(src.as_raw(), src_size, dst_size, 16, alg));
            *src = ImageBuffer::from_raw(dst_width, dst_height, resized).unwrap();
        }
        DynamicImage::ImageRgba16(src) => {
            let resized = wm_try!(resize_rgba16(src.as_raw(), src_size, dst_size, 16, alg));
            *src = ImageBuffer::from_raw(dst_width, dst_height, resized).unwrap();
        }
        DynamicImage::ImageRgb32F(src) => {
            let resized = wm_try!(resize_rgb_f32(src.as_raw(), src_size, dst_size, alg));
            *src = ImageBuffer::from_raw(dst_width, dst_height, resized).unwrap();
        }
        DynamicImage::ImageRgba32F(src) => {
            let resized = wm_try!(resize_rgba_f32(src.as_raw(), src_size, dst_size, alg));
            *src = ImageBuffer::from_raw(dst_width, dst_height, resized).unwrap();
        }
        _ => unreachable!(),
    }
    if premultiplied_by_alpha {
        unpremultiply_alpha(image);
    }
    Ok(())
}

/// Return value indicates whether the image was in premultiplied by alpha
#[must_use]
fn premultiply_alpha_if_needed(image: &mut DynamicImage) -> bool {
    use pic_scale_safe::*;
    if !has_constant_alpha(image) {
        match image {
            DynamicImage::ImageLuma8(_) => false,
            DynamicImage::ImageLumaA8(buf) => {
                premultiply_la8(buf.as_mut());
                true
            }
            DynamicImage::ImageRgb8(_) => false,
            DynamicImage::ImageRgba8(buf) => {
                premultiply_rgba8(buf.as_mut());
                true
            }
            DynamicImage::ImageLuma16(_) => false,
            DynamicImage::ImageLumaA16(buf) => {
                premultiply_la16(buf.as_mut(), 16);
                true
            }
            DynamicImage::ImageRgb16(_) => false,
            DynamicImage::ImageRgba16(buf) => {
                premultiply_rgba16(buf.as_mut(), 16);
                true
            }
            DynamicImage::ImageRgb32F(_) => false,
            DynamicImage::ImageRgba32F(buf) => {
                premultiply_rgba_f32(buf.as_mut());
                true
            }
            _ => unreachable!(),
        }
    } else {
        false
    }
}

/// Reverses premultiplication by alpha
fn unpremultiply_alpha(image: &mut DynamicImage) {
    use pic_scale_safe::*;
    match image {
        DynamicImage::ImageLuma8(_) => (),
        DynamicImage::ImageLumaA8(buf) => unpremultiply_la8(buf.as_mut()),
        DynamicImage::ImageRgb8(_) => (),
        DynamicImage::ImageRgba8(buf) => unpremultiply_rgba8(buf.as_mut()),
        DynamicImage::ImageLuma16(_) => (),
        DynamicImage::ImageLumaA16(buf) => unpremultiply_la16(buf.as_mut(), 16),
        DynamicImage::ImageRgb16(_) => (),
        DynamicImage::ImageRgba16(buf) => unpremultiply_rgba16(buf.as_mut(), 16),
        DynamicImage::ImageRgb32F(_) => (),
        DynamicImage::ImageRgba32F(buf) => unpremultiply_rgba_f32(buf),
        _ => todo!(),
    }
}

#[must_use]
fn has_constant_alpha(image: &DynamicImage) -> bool {
    match image {
        DynamicImage::ImageLuma8(_) => true,
        DynamicImage::ImageLumaA8(buf) => has_constant_alpha_integer(buf),
        DynamicImage::ImageRgb8(_) => true,
        DynamicImage::ImageRgba8(buf) => has_constant_alpha_integer(buf),
        DynamicImage::ImageLuma16(_) => true,
        DynamicImage::ImageLumaA16(buf) => has_constant_alpha_integer(buf),
        DynamicImage::ImageRgb16(_) => true,
        DynamicImage::ImageRgba16(buf) => has_constant_alpha_integer(buf),
        DynamicImage::ImageRgb32F(_) => true,
        DynamicImage::ImageRgba32F(buf) => has_constant_alpha_f32(buf),
        _ => unreachable!(),
    }
}

#[must_use]
fn has_constant_alpha_integer<P, Container>(img: &ImageBuffer<P, Container>) -> bool
where
    P: Pixel + 'static,
    Container: std::ops::Deref<Target = [P::Subpixel]>,
    P::Subpixel:
        Copy + PartialEq + BitXor<P::Subpixel, Output = P::Subpixel> + AddAssign + Into<u64>,
{
    let first_pixel_alpha = *match img.pixels().next() {
        Some(pixel) => pixel.channels().last().unwrap(), // there doesn't seem to be a better way to retrieve the alpha channel
        None => return true,                             // empty input image
    };
    // A naive, slower implementation that branches on every pixel:
    //
    // img.pixels().map(|pixel| pixel.channels().last().unwrap()).all(|alpha| alpha == first_pixel_alpha);
    //
    // Instead of doing that we scan every row first with cheap arithmetic instructions
    // and only compare the sum of divergences on every row, which should be 0
    let mut sum_of_diffs: u64 = 0;
    for row in img.rows() {
        row.for_each(|pixel| {
            let alpha = pixel.alpha();
            sum_of_diffs += alpha.bitxor(first_pixel_alpha).into();
        });
        if sum_of_diffs != 0 {
            return false;
        }
    }
    true
}

#[must_use]
fn has_constant_alpha_f32(img: &ImageBuffer<image::Rgba<f32>, Vec<f32>>) -> bool {
    // Optimizing correctly in presence of NaNs and infinities is tricky, so just do the naive thing for now
    let first_pixel_alpha = match img.pixels().next() {
        Some(pixel) => pixel.alpha(),
        None => return true,                             // empty input image
    };
    img.pixels()
        .map(|pixel| pixel.channels().last().unwrap())
        .all(|alpha| *alpha == first_pixel_alpha)
}

#[must_use]
fn compute_dimensions(image: &DynamicImage, geometry: &ResizeGeometry) -> (u32, u32) {
    let constraint = geometry.constraint;
    match geometry.target {
        ResizeTarget::Size {
            width,
            height,
            ignore_aspect_ratio,
        } => {
            if ignore_aspect_ratio {
                let width = compute_dimension(image.width(), width, &constraint);
                let height = compute_dimension(image.height(), height, &constraint);
                (width, height)
            } else {
                preserve_aspect_ratio(image, width, height, &constraint)
            }
        }
        ResizeTarget::Percentage { width, height } => {
            // return early on a no-op
            if height == 100.0 && (width.is_none() || width == Some(100.0)) {
                (image.width(), image.height())
            } else {
                let width = match width {
                    Some(percent) => apply_percentage(image.width(), percent),
                    None => image.width(),
                };
                let height = apply_percentage(image.height(), height);
                (width, height)
            }
        }
        ResizeTarget::Area(area) => {
            let (width, height) = size_with_max_area(image.width(), image.height(), area);
            match constraint {
                ResizeConstraint::Unconstrained => (width, height),
                ResizeConstraint::OnlyEnlarge => {
                    (width.max(image.width()), height.max(image.height()))
                }
                ResizeConstraint::OnlyShrink => {
                    (width.min(image.width()), height.min(image.height()))
                }
            }
        }
        ResizeTarget::FullyCover { width, height } => cover_area(image, width, height),
    }
}

#[must_use]
/// Scale the image dimension by the given percentage
fn apply_percentage(size: u32, percentage: f64) -> u32 {
    // dividing by 100 at the *end* minimizes precision loss
    (size as f64 * percentage / 100.0).round() as u32
}

#[must_use]
fn compute_dimension(
    image_size: u32,
    target_size: Option<u32>,
    constraint: &ResizeConstraint,
) -> u32 {
    // If no size is specified for this dimension, keep the image's original size
    let target_size = target_size.unwrap_or(image_size);

    let size = match constraint {
        ResizeConstraint::Unconstrained => target_size,
        ResizeConstraint::OnlyEnlarge => image_size.max(target_size),
        ResizeConstraint::OnlyShrink => image_size.min(target_size),
    };

    // imagemagick emits a 1x1 image if you ask for a 0x0 one
    prevent_zero(size)
}

#[must_use]
/// Returns `(width, height)`
fn preserve_aspect_ratio(
    image: &DynamicImage,
    target_width: Option<u32>,
    target_height: Option<u32>,
    constraint: &ResizeConstraint,
) -> (u32, u32) {
    assert!(target_width.is_some() || target_height.is_some());
    let target_width = compute_dimension(
        image.width(),
        Some(target_width.unwrap_or(u32::MAX)),
        constraint,
    );
    let target_height = compute_dimension(
        image.height(),
        Some(target_height.unwrap_or(u32::MAX)),
        constraint,
    );
    let image_ratio = Fraction::new(image.width(), image.height());
    let target_ratio = Fraction::new(target_width, target_height);
    use std::cmp::Ordering;
    match image_ratio.cmp(&target_ratio) {
        Ordering::Less => {
            // the image is narrower than the target dimensions, reduce width
            let mut width = (image_ratio.to_float() * target_height as f64).round() as u32;
            width = prevent_zero(width);
            (width, target_height)
        }
        Ordering::Greater => {
            // the image is wider than the target dimensions, reduce height
            let mut height =
                (image_ratio.reciprocal().to_float() * target_width as f64).round() as u32;
            height = prevent_zero(height);
            (target_width, height)
        }
        Ordering::Equal => (target_width, target_height),
    }
}

#[must_use]
/// Almost a carbon copy of `preserve_aspect_ratio()`, but fits to cover the whole area
/// instead of fitting inside it.
/// Returns `(width, height)`
fn cover_area(image: &DynamicImage, target_width: u32, target_height: u32) -> (u32, u32) {
    // Literally the only implementation difference from preserve_aspect_ratio is the swapped contents
    // of Ordering::Less and Ordering::Greater branches
    let image_ratio = Fraction::new(image.width(), image.height());
    let target_ratio = Fraction::new(target_width, target_height);
    use std::cmp::Ordering;
    match image_ratio.cmp(&target_ratio) {
        Ordering::Greater => {
            let mut width = (image_ratio.to_float() * target_height as f64).round() as u32;
            width = prevent_zero(width);
            (width, target_height)
        }
        Ordering::Less => {
            let mut height =
                (image_ratio.reciprocal().to_float() * target_width as f64).round() as u32;
            height = prevent_zero(height);
            (target_width, height)
        }
        Ordering::Equal => (target_width, target_height),
    }
}

#[must_use]
fn prevent_zero(size: u32) -> u32 {
    if size == 0 {
        1
    } else {
        size
    }
}

#[must_use]
fn size_with_max_area(width: u32, height: u32, max_area: u64) -> (u32, u32) {
    let original_area = (width as u64) * (height as u64);
    let scale_factor = (max_area as f64 / original_area as f64).sqrt();

    // We do not .round() here to avoid accidentally exceeding the allotted area.
    // Casting via `as` will always round down, which is what we want.
    let new_width = (width as f64 * scale_factor) as u32;
    let new_height = (height as f64 * scale_factor) as u32;
    // I have verified that this hold up to 16TB in area with a fuzzer
    debug_assert!(new_width as u64 * new_height as u64 <= max_area);
    (new_width, new_height)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn preserve_aspect_ratio_wide() {
        let image = DynamicImage::new_rgb8(800, 600);
        assert_eq!(
            preserve_aspect_ratio(
                &image,
                Some(100),
                Some(100),
                &ResizeConstraint::Unconstrained
            ),
            (100, 75)
        );
    }

    #[test]
    fn preserve_aspect_ratio_wide_upscale() {
        let image = DynamicImage::new_rgb8(100, 75);
        assert_eq!(
            preserve_aspect_ratio(
                &image,
                Some(800),
                Some(800),
                &ResizeConstraint::Unconstrained
            ),
            (800, 600)
        );
    }

    #[test]
    fn preserve_aspect_ratio_narrow() {
        let image = DynamicImage::new_rgb8(600, 800);
        assert_eq!(
            preserve_aspect_ratio(
                &image,
                Some(100),
                Some(100),
                &ResizeConstraint::Unconstrained
            ),
            (75, 100)
        );
    }

    #[test]
    fn preserve_aspect_ratio_narrow_upscale() {
        let image = DynamicImage::new_rgb8(75, 100);
        assert_eq!(
            preserve_aspect_ratio(
                &image,
                Some(800),
                Some(800),
                &ResizeConstraint::Unconstrained
            ),
            (600, 800)
        );
    }

    #[test]
    fn preserve_aspect_ratio_same() {
        let image = DynamicImage::new_rgb8(800, 800);
        assert_eq!(
            preserve_aspect_ratio(
                &image,
                Some(100),
                Some(100),
                &ResizeConstraint::Unconstrained
            ),
            (100, 100)
        );
    }

    #[test]
    fn preserve_aspect_ratio_width_only() {
        let image = DynamicImage::new_rgb8(64, 100);
        let geometry = ResizeGeometry::from_str("200").unwrap();
        assert_eq!((200, 313), compute_dimensions(&image, &geometry));
    }

    #[test]
    fn preserve_aspect_ratio_height_only() {
        let image = DynamicImage::new_rgb8(64, 100);
        let geometry = ResizeGeometry::from_str("x200").unwrap();
        assert_eq!((128, 200), compute_dimensions(&image, &geometry));
    }

    #[test]
    fn preserve_aspect_ratio_enlarge_only() {
        let image = DynamicImage::new_rgb8(100, 100);
        let geometry = ResizeGeometry::from_str("120x120<").unwrap();
        assert_eq!((120, 120), compute_dimensions(&image, &geometry));
    }

    #[test]
    fn preserve_aspect_ratio_enlarge_only_no_op() {
        let image = DynamicImage::new_rgb8(100, 100);
        let geometry = ResizeGeometry::from_str("50x50<").unwrap();
        assert_eq!((100, 100), compute_dimensions(&image, &geometry));
    }

    #[test]
    fn preserve_aspect_ratio_shrink_only() {
        let image = DynamicImage::new_rgb8(120, 120);
        let geometry = ResizeGeometry::from_str("100x100>").unwrap();
        assert_eq!((100, 100), compute_dimensions(&image, &geometry));
    }

    #[test]
    fn preserve_aspect_ratio_shrink_only_no_op() {
        let image = DynamicImage::new_rgb8(50, 50);
        let geometry = ResizeGeometry::from_str("100x100>").unwrap();
        assert_eq!((50, 50), compute_dimensions(&image, &geometry));
    }

    #[test]
    fn preserve_aspect_ratio_shrink_only_by_width() {
        let image = DynamicImage::new_rgb8(100, 50);
        let geometry = ResizeGeometry::from_str("50x50>").unwrap();
        assert_eq!((50, 25), compute_dimensions(&image, &geometry));
    }

    #[test]
    fn preserve_aspect_ratio_shrink_only_by_height() {
        let image = DynamicImage::new_rgb8(50, 100);
        let geometry = ResizeGeometry::from_str("50x50>").unwrap();
        assert_eq!((25, 50), compute_dimensions(&image, &geometry));
    }

    #[test]
    fn percentage() {
        let image = DynamicImage::new_rgb8(800, 600);
        let geometry = ResizeGeometry::from_str("50%").unwrap();
        assert_eq!((400, 300), compute_dimensions(&image, &geometry));
    }

    #[test]
    fn height_percentage() {
        let image = DynamicImage::new_rgb8(800, 600);
        let geometry = ResizeGeometry::from_str("x50%").unwrap();
        assert_eq!((800, 300), compute_dimensions(&image, &geometry));
    }

    #[test]
    fn fractional_percentage() {
        let image = DynamicImage::new_rgb8(1000, 1000);
        let geometry = ResizeGeometry::from_str("4.5%").unwrap();
        assert_eq!((45, 45), compute_dimensions(&image, &geometry));
    }

    #[test]
    fn different_percentages() {
        let image = DynamicImage::new_rgb8(1000, 1000);
        let geometry = ResizeGeometry::from_str("4x30%").unwrap();
        assert_eq!((40, 300), compute_dimensions(&image, &geometry));
    }

    #[test]
    fn max_area() {
        let computed = size_with_max_area(100, 100, 900);
        assert_eq!((30, 30), computed);
    }

    #[test]
    fn max_area_unconstrained() {
        let image = DynamicImage::new_rgb8(100, 100);
        let geometry = ResizeGeometry::from_str("900@").unwrap();
        assert_eq!((30, 30), compute_dimensions(&image, &geometry));
    }

    #[test]
    fn max_area_enlarge_only() {
        let image = DynamicImage::new_rgb8(100, 100);
        let geometry = ResizeGeometry::from_str("900@<").unwrap();
        assert_eq!((100, 100), compute_dimensions(&image, &geometry));
    }

    #[test]
    fn max_area_shrink_only() {
        let image = DynamicImage::new_rgb8(100, 100);
        let geometry = ResizeGeometry::from_str("900@>").unwrap();
        assert_eq!((30, 30), compute_dimensions(&image, &geometry));
    }

    #[test]
    fn cover_area_width() {
        let image = DynamicImage::new_rgb8(200, 150);
        let geometry = ResizeGeometry::from_str("100^").unwrap();
        assert_eq!((133, 100), compute_dimensions(&image, &geometry));
    }

    #[test]
    fn cover_area_height() {
        let image = DynamicImage::new_rgb8(150, 200);
        let geometry = ResizeGeometry::from_str("100^").unwrap();
        assert_eq!((100, 133), compute_dimensions(&image, &geometry));
    }

    #[test]
    fn cover_area_width_upscale() {
        let image = DynamicImage::new_rgb8(50, 25);
        let geometry = ResizeGeometry::from_str("100^").unwrap();
        assert_eq!((200, 100), compute_dimensions(&image, &geometry));
    }

    #[test]
    fn cover_area_height_upscale() {
        let image = DynamicImage::new_rgb8(25, 50);
        let geometry = ResizeGeometry::from_str("100^").unwrap();
        assert_eq!((100, 200), compute_dimensions(&image, &geometry));
    }
}
