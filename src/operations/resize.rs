use fast_image_resize::{ResizeAlg, ResizeOptions, Resizer};
use image::DynamicImage;

use crate::arg_parsers::{ResizeConstraint, ResizeGeometry};

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
    let width = compute_dimension(image.width(), geometry.width, geometry);
    let height = compute_dimension(image.height(), geometry.height, geometry);
    (width, height)
}

fn compute_dimension(image_size: u32, geom_size: Option<u32>, geometry: &ResizeGeometry) -> u32 {
    // If no size is specified for this dimension, keep the image's original size
    let geom_size = geom_size.unwrap_or(image_size);

    let size = match geometry.constraint {
        ResizeConstraint::Any => geom_size,
        ResizeConstraint::OnlyEnlarge => image_size.max(geom_size),
        ResizeConstraint::OnlyShrink => image_size.min(geom_size),
    };

    // imagemagick emits a 1x1 image if you ask for a 0x0 one
    if size == 0 {
        1
    } else {
        size
    }
}
