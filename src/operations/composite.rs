use crate::{error::MagickError, image::Image};
use image::{imageops::overlay, Rgba};
use imageproc::map::map_pixels_mut;

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

pub struct Alpha(pub f32);

pub fn composite(
    image1: &mut Image,
    image2: &mut Image,
    gravity: Gravity,
    alpha: Alpha,
) -> Result<(), MagickError> {
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

    if let Some(rgba_img) = image2.pixels.as_mut_rgba8() {
        map_pixels_mut(rgba_img, |_x, _y, pixel| {
            let a = pixel[3];
            Rgba([pixel[0], pixel[1], pixel[2], ((a as f32) * alpha.0).min(255.0) as u8])
        });
    }

    overlay(&mut image1.pixels, &image2.pixels, x, y);
    Ok(())
}
