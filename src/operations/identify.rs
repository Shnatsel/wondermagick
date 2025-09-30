use crate::{error::MagickError, image::Image};

pub fn identify(image: &mut Image) -> Result<(), MagickError> {
    println!("{}x{}", image.pixels.width(), image.pixels.height(),);
    Ok(())
}
