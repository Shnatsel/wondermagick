use crate::{
    arg_parsers::{FileFormat, Location},
    decode::decode,
    error::MagickError,
    image::Image,
};
use image::imageops::overlay;

pub fn composite(
    image1: &mut Image,
    image2_location: &Location,
    image2_format: Option<FileFormat>,
    //TODO: If a third image is given this is treated as a grayscale blending 'mask' image
    // relative to the first 'destination' image. This mask is blended with the source image.
) -> Result<(), MagickError> {
    let image2 = decode(image2_location, image2_format)?;
    overlay(&mut image1.pixels, &image2.pixels, 0, 0);
    Ok(())
}
