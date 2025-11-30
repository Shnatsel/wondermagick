use crate::{error::MagickError, image::Image};

#[derive(Debug, Clone, PartialEq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

pub fn flip(image: &mut Image, axis: &Axis) -> Result<(), MagickError> {
    match axis {
        Axis::Horizontal => {
            image::imageops::flip_horizontal_in_place(&mut image.pixels);
            Ok(())
        }
        Axis::Vertical => {
            image::imageops::flip_vertical_in_place(&mut image.pixels);
            Ok(())
        }
    }
}
