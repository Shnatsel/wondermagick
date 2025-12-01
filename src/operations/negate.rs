use crate::{error::MagickError, image::Image};

pub fn negate(image: &mut Image) -> Result<(), MagickError> {
    image.pixels.invert();
    Ok(())
}
