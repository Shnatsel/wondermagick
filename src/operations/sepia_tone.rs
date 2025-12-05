use crate::{arg_parsers::SepiaThreshold, error::MagickError, image::Image};

pub fn sepia_tone(image: &mut Image, _threshold: &SepiaThreshold) -> Result<(), MagickError> {
    let mut rbga8 = image.pixels.to_rgba8();
    for pixel in rbga8.pixels_mut() {
        let r = pixel[0] as f32;
        let g = pixel[1] as f32;
        let b = pixel[2] as f32;

        // German wikipedia shows coefficients for digital sepia toning.
        // https://de.wikipedia.org/wiki/Sepia_(Fotografie)
        pixel[0] = (0.393 * r + 0.769 * g + 0.189 * b).min(255.0) as u8;
        pixel[1] = (0.349 * r + 0.686 * g + 0.168 * b).min(255.0) as u8;
        pixel[2] = (0.272 * r + 0.534 * g + 0.131 * b).min(255.0) as u8;
    }
    image.pixels = image::DynamicImage::ImageRgba8(rbga8);
    Ok(())
}
