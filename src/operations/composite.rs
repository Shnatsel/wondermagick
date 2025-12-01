use crate::{
    arg_parsers::{FileFormat, Gravity, Location},
    decode::decode,
    error::MagickError,
    image::Image,
};
use image::{imageops::overlay, GenericImageView};

pub fn composite(
    image1: &mut Image,
    image2_location: &Location,
    image2_format: Option<FileFormat>,
    gravity: Option<&Gravity>,
    // TODO: If a third image is given this is treated as a grayscale blending 'mask' image
    // relative to the first 'destination' image. This mask is blended with the source image.
) -> Result<(), MagickError> {
    let image2 = decode(image2_location, image2_format)?;
    let (x, y) = if let Some(g) = gravity {
        offset_from_gravity(g, image1.pixels.dimensions(), image2.pixels.dimensions())
    } else {
        (0, 0)
    };
    overlay(&mut image1.pixels, &image2.pixels, x as i64, y as i64);
    Ok(())
}

fn offset_from_gravity(
    gravity: &Gravity,
    image1_dim: (u32, u32),
    image2_dim: (u32, u32),
) -> (u32, u32) {
    match gravity {
        Gravity::Center => (
            (image1_dim.0 - image2_dim.0) / 2,
            (image1_dim.1 - image2_dim.1) / 2,
        ),
        Gravity::North => ((image1_dim.0 - image2_dim.0) / 2, 0),
        Gravity::South => (
            (image1_dim.0 - image2_dim.0) / 2,
            image1_dim.1 - image2_dim.1,
        ),
        Gravity::East => (
            image1_dim.0 - image2_dim.0,
            (image1_dim.1 - image2_dim.1) / 2,
        ),
        Gravity::West => (0, (image1_dim.1 - image2_dim.1) / 2),
        Gravity::NorthEast => (image1_dim.0 - image2_dim.0, 0),
        Gravity::NorthWest => (0, 0),
        Gravity::SouthEast => (image1_dim.0 - image2_dim.0, image1_dim.1 - image2_dim.1),
        Gravity::SouthWest => (0, image1_dim.1 - image2_dim.1),
    }
}
