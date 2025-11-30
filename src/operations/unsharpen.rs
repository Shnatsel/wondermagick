use crate::{arg_parsers::UnsharpenGeometry, error::MagickError, image::Image};

pub fn unsharpen(image: &mut Image, geom: &UnsharpenGeometry) -> Result<(), MagickError> {
    image.pixels = image.pixels.unsharpen(geom.sigma, geom.threshold);
    Ok(())
}
