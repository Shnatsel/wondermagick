use crate::{arg_parsers::BlurGeometry, arg_parsers::Sigma, error::MagickError, image::Image};

pub fn blur(image: &mut Image, geometry: &BlurGeometry) -> Result<(), MagickError> {
    let Sigma(sigma) = geometry.sigma;
    image.pixels = image.pixels.blur(sigma);
    Ok(())
}
