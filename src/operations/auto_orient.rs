use image::metadata::Orientation;

use crate::{error::MagickError, image::Image};

pub fn auto_orient(image: &mut Image) -> Result<(), MagickError> {
    if let Some(exif) = &mut image.exif {
        let orientation = Orientation::remove_from_exif_chunk(exif);
        if let Some(orientation) = orientation {
            image.pixels.apply_orientation(orientation);
        }
    }
    Ok(())
}
