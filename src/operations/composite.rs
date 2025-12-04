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
    gravity: Option<Gravity>,
    // TODO: If a third image is given this is treated as a grayscale blending 'mask' image
    // relative to the first 'destination' image. This mask is blended with the source image.
) -> Result<(), MagickError> {
    let image2 = decode(image2_location, image2_format)?;
    let (x, y) = if let Some(g) = gravity {
        offset_from_gravity(&g, image1.pixels.dimensions(), image2.pixels.dimensions())
    } else {
        (0, 0)
    };
    overlay(&mut image1.pixels, &image2.pixels, x, y);
    Ok(())
}

fn offset_from_gravity(
    gravity: &Gravity,
    image1_dim: (u32, u32),
    image2_dim: (u32, u32),
) -> (i64, i64) {
    eprintln!(
        "Calculating offset for gravity {:?} with image1_dim {:?} and image2_dim {:?}",
        gravity, image1_dim, image2_dim
    );
    let w1 = i64::from(image1_dim.0);
    let h1 = i64::from(image1_dim.1);
    let w2 = i64::from(image2_dim.0);
    let h2 = i64::from(image2_dim.1);

    match gravity {
        Gravity::Center => ((w1 - w2) / 2, (h1 - h2) / 2),
        Gravity::North => ((w1 - w2) / 2, 0),
        Gravity::South => ((w1 - w2) / 2, h1 - h2),
        Gravity::East => (w1 - w2, (h1 - h2) / 2),
        Gravity::West => (0, (h1 - h2) / 2),
        Gravity::NorthEast => (w1 - w2, 0),
        Gravity::NorthWest | Gravity::Forget | Gravity::None => (0, 0),
        Gravity::SouthEast => (w1 - w2, h1 - h2),
        Gravity::SouthWest => (0, h1 - h2),
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
            Gravity::None,
            Gravity::Forget,
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
            (0, 0),
            (0, 0),
            (600, 500),
            (0, 500)
        }
    )]
    fn test_offset_from_gravity(gravity: Gravity, expected_offsets: (i64, i64)) {
        assert_eq!(
            offset_from_gravity(&gravity, IMG1_DIM, IMG2_DIM),
            expected_offsets
        );
    }

    #[test]
    fn test_offset_from_gravity_img2_larger_than_img1() {
        let img1_dim = (100, 100);
        let img2_dim = (200, 200);
        let offsets = offset_from_gravity(&Gravity::Center, img1_dim, img2_dim);
        assert_eq!(offsets, (-50, -50));
    }
}
