use fast_image_resize::{ResizeAlg, ResizeOptions, Resizer};
use image::DynamicImage;

use crate::{
    arg_parsers::{ResizeConstraint, ResizeGeometry},
    utils::fraction::Fraction,
};

pub fn resize(image: &mut DynamicImage, geometry: &ResizeGeometry) {
    let (dst_width, dst_height) = compute_dimensions(image, geometry);
    resize_impl(image, dst_width, dst_height, Default::default())
}

pub fn thumbnail(image: &mut DynamicImage, geometry: &ResizeGeometry) {
    let (dst_width, dst_height) = compute_dimensions(image, geometry);

    // imagemagick first downscales to 5x the target size with the cheap nearest-neighbor algorithm
    let width = image.width().min(dst_width * 5);
    let height = image.height().min(dst_height * 5);
    resize_impl(image, width, height, ResizeAlg::Nearest);

    // now do the actual resize to the target dimensions
    resize_impl(image, dst_width, dst_height, Default::default());
}

fn resize_impl(image: &mut DynamicImage, dst_width: u32, dst_height: u32, algorithm: ResizeAlg) {
    if image.width() == dst_width && image.height() == dst_height {
        return;
    }
    let mut resizer = Resizer::new(); // TODO: cache the resizer
    let mut dst_image = DynamicImage::new(dst_width, dst_height, image.color());
    let options = ResizeOptions::default().resize_alg(algorithm);
    resizer
        .resize(image, &mut dst_image, Some(&options))
        .unwrap();
    *image = dst_image;
}

fn compute_dimensions(image: &DynamicImage, geometry: &ResizeGeometry) -> (u32, u32) {
    let mut width = compute_dimension(image.width(), geometry.width, geometry);
    let mut height = compute_dimension(image.height(), geometry.height, geometry);
    if !geometry.ignore_aspect_ratio {
        (width, height) = preserve_aspect_ratio(image, width, height);
    }
    (width, height)
}

fn compute_dimension(image_size: u32, target_size: Option<u32>, geometry: &ResizeGeometry) -> u32 {
    // If no size is specified for this dimension, keep the image's original size
    let target_size = target_size.unwrap_or(image_size);

    let size = match geometry.constraint {
        ResizeConstraint::Any => target_size,
        ResizeConstraint::OnlyEnlarge => image_size.max(target_size),
        ResizeConstraint::OnlyShrink => image_size.min(target_size),
    };

    // imagemagick emits a 1x1 image if you ask for a 0x0 one
    prevent_zero(size)
}

/// Returns `(width, height)`
fn preserve_aspect_ratio(
    image: &DynamicImage,
    target_width: u32,
    target_height: u32,
) -> (u32, u32) {
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

fn prevent_zero(size: u32) -> u32 {
    if size == 0 {
        1
    } else {
        size
    }
}

#[cfg(test)]
mod tests {
    use image::DynamicImage;

    use super::preserve_aspect_ratio;

    #[test]
    fn preserve_aspect_ratio_wide() {
        let image = DynamicImage::new_rgb8(800, 600);
        assert_eq!(preserve_aspect_ratio(&image, 100, 100), (100, 75));
    }

    #[test]
    fn preserve_aspect_ratio_wide_upscale() {
        let image = DynamicImage::new_rgb8(100, 75);
        assert_eq!(preserve_aspect_ratio(&image, 800, 800), (800, 600));
    }

    #[test]
    fn preserve_aspect_ratio_narrow() {
        let image = DynamicImage::new_rgb8(600, 800);
        assert_eq!(preserve_aspect_ratio(&image, 100, 100), (75, 100));
    }

    #[test]
    fn preserve_aspect_ratio_narrow_upscale() {
        let image = DynamicImage::new_rgb8(75, 100);
        assert_eq!(preserve_aspect_ratio(&image, 800, 800), (600, 800));
    }

    #[test]
    fn preserve_aspect_ratio_same() {
        let image = DynamicImage::new_rgb8(800, 800);
        assert_eq!(preserve_aspect_ratio(&image, 100, 100), (100, 100));
    }
}
