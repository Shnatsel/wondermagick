use crate::{arg_parsers::SepiaThreshold, error::MagickError, image::Image};

pub fn sepia_tone(image: &mut Image, _threshold: &SepiaThreshold) -> Result<(), MagickError> {
    let mut rbga = image.pixels.to_rgba32f();
    for pixel in rbga.pixels_mut() {
        let r = pixel[0];
        let g = pixel[1];
        let b = pixel[2];

        // German wikipedia shows coefficients for digital sepia toning.
        // https://de.wikipedia.org/wiki/Sepia_(Fotografie)
        pixel[0] = (0.393 * r + 0.769 * g + 0.189 * b).min(255.0);
        pixel[1] = (0.349 * r + 0.686 * g + 0.168 * b).min(255.0);
        pixel[2] = (0.272 * r + 0.534 * g + 0.131 * b).min(255.0);
    }
    image.pixels = image::DynamicImage::ImageRgba32F(rbga);
    Ok(())
}
