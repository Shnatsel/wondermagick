use fast_image_resize::Resizer;
use image::DynamicImage;

use crate::arg_parsers::ResizeGeometry;

pub fn resize(image: &mut DynamicImage, geometry: &ResizeGeometry) {
    let (dst_width, dst_height) = compute_dimensions(image, geometry);
    if image.width() == dst_width && image.height() == dst_height {
        return;
    }
    let mut resizer = Resizer::new(); // TODO: cache the resizer
    let mut dst_image = DynamicImage::new(dst_width, dst_height, image.color());
    resizer.resize(image, &mut dst_image, None).unwrap();
    *image = dst_image;
}

fn compute_dimensions(image: &DynamicImage, geometry: &ResizeGeometry) -> (u32, u32) {
    let width = compute_dimension(image.width(), geometry.width, geometry);
    let height = compute_dimension(image.height(), geometry.height, geometry);
    (width, height)
}

fn compute_dimension(image_size: u32, geom_size: Option<u32>, _geometry: &ResizeGeometry) -> u32 {
    // TODO: do the actual computation accounting for flags
    geom_size.unwrap_or(image_size)
}
