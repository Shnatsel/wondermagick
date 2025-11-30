use crate::{arg_parsers::UnsharpGeometry, error::MagickError, image::Image};

pub fn unsharp(image: &mut Image, geom: &UnsharpGeometry) -> Result<(), MagickError> {
    image.pixels = image.pixels.unsharpen(geom.sigma, geom.threshold);
    Ok(())
}
