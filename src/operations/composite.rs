use crate::{error::MagickError, image::Image};
use image::imageops::overlay;

pub fn composite(image1: &mut Image, image2: &Image) -> Result<(), MagickError> {
    overlay(&mut image1.pixels, &image2.pixels, 0, 0);
    Ok(())
}
