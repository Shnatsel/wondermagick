use crate::{error::MagickError, image::Image};

pub fn blur(image: &mut Image) -> Result<(), MagickError> {
    image.pixels = image.pixels.blur(5.0);
    Ok(())
}
