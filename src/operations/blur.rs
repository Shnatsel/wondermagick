use crate::{arg_parsers::BlurGeometry, error::MagickError, image::Image};

pub fn blur(image: &mut Image, geometry: &BlurGeometry) -> Result<(), MagickError> {
    image.pixels = image.pixels.fast_blur(geometry.sigma);
    Ok(())
}
