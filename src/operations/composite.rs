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

#[cfg(test)]
mod tests {
    use super::offset_from_gravity;
    use super::Gravity;
    use parameterized::parameterized;

    const IMG1_DIM: (u32, u32) = (800, 600);
    const IMG2_DIM: (u32, u32) = (200, 100);

    #[parameterized(
        gravity = {
            Gravity::Center,
            Gravity::North,
            Gravity::South,
            Gravity::East,
            Gravity::West,
            Gravity::NorthEast,
            Gravity::NorthWest,
            Gravity::SouthEast,
            Gravity::SouthWest
        },
        expected_offsets = {
            (300, 250),
            (300, 0),
            (300, 500),
            (600, 250),
            (0, 250),
            (600, 0),
            (0, 0),
            (600, 500),
            (0, 500)
        }
    )]
    fn test_offset_from_gravity(gravity: Gravity, expected_offsets: (u32, u32)) {
        assert_eq!(
            offset_from_gravity(&gravity, IMG1_DIM, IMG2_DIM),
            expected_offsets
        );
    }
}
