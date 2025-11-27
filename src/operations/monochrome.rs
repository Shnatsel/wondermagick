use crate::{error::MagickError, image::Image};
use image::DynamicImage;
use imageproc::contrast::{otsu_level, threshold, ThresholdType};

pub fn monochrome(image: &mut Image) -> Result<(), MagickError> {
    let grayscale = image.pixels.to_luma8();

    image.pixels = DynamicImage::ImageLuma8(threshold(
        &grayscale,
        otsu_level(&grayscale),
        ThresholdType::Binary,
    ));

    Ok(())
}
