use crate::{error::MagickError, image::Image};
use image::imageops::{flip_horizontal_in_place, flip_vertical_in_place};

#[derive(Debug, Clone, PartialEq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

pub fn flip(image: &mut Image, axis: &Axis) -> Result<(), MagickError> {
    match axis {
        Axis::Horizontal => flip_horizontal_in_place(&mut image.pixels),
        Axis::Vertical => flip_vertical_in_place(&mut image.pixels),
    };
    Ok(())
}
