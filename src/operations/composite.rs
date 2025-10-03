use crate::{error::MagickError, image::Image};
use image::imageops::overlay;

pub enum Gravity {
    Center,
    North,
    South,
    East,
    West,
    Northeast,
    Northwest,
    Southeast,
    Southwest,
}

pub fn composite(image1: &mut Image, image2: &Image, gravity: Gravity) -> Result<(), MagickError> {
    let (x, y) = match gravity {
        Gravity::Center => (
            (image1.pixels.width() as i64 - image2.pixels.width() as i64) / 2,
            (image1.pixels.height() as i64 - image2.pixels.height() as i64) / 2,
        ),
        Gravity::North => (
            (image1.pixels.width() as i64 - image2.pixels.width() as i64) / 2,
            0,
        ),
        Gravity::South => (
            (image1.pixels.width() as i64 - image2.pixels.width() as i64) / 2,
            image1.pixels.height() as i64 - image2.pixels.height() as i64,
        ),
        Gravity::East => (
            image1.pixels.width() as i64 - image2.pixels.width() as i64,
            (image1.pixels.height() as i64 - image2.pixels.height() as i64) / 2,
        ),
        Gravity::West => (
            0,
            (image1.pixels.height() as i64 - image2.pixels.height() as i64) / 2,
        ),
        Gravity::Northeast => (
            image1.pixels.width() as i64 - image2.pixels.width() as i64,
            0,
        ),
        Gravity::Northwest => (0, 0),
        Gravity::Southeast => (
            image1.pixels.width() as i64 - image2.pixels.width() as i64,
            image1.pixels.height() as i64 - image2.pixels.height() as i64,
        ),
        Gravity::Southwest => (
            0,
            image1.pixels.height() as i64 - image2.pixels.height() as i64,
        ),
    };

    overlay(&mut image1.pixels, &image2.pixels, x, y);
    Ok(())
}
