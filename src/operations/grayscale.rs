use crate::{arg_parsers::GrayscaleMethod, error::MagickError, image};

pub fn grayscale(image: &mut image::Image, method: &GrayscaleMethod) -> Result<(), MagickError> {
    Ok(())
}
