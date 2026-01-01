use crate::{error::MagickError, image::Image};

pub fn noise(image: &mut Image) -> Result<(), MagickError> {
    let mut rgba_image = image.pixels.to_rgba8();
    imageproc::noise::gaussian_noise_mut(&mut rgba_image, 20.0, 15.0, 10);
    image.pixels = image::DynamicImage::ImageRgba8(rgba_image);
    Ok(())
}
