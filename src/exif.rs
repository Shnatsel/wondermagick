use exif::{In, Tag};
use image::{DynamicImage, Orientation};

use crate::{error::MagickError, wm_err, wm_try};

/// Parses the provided Exif chunk to extract the orientation and rotates the image accordingly
pub fn rotate_by_exif(image: &mut DynamicImage, raw_exif: Vec<u8>) -> Result<(), MagickError> {
    let reader = exif::Reader::new();
    let exif = wm_try!(reader.read_raw(raw_exif));
    // based on the Exif crate example: https://docs.rs/kamadak-exif/latest/exif/index.html#examples
    if let Some(orientation) = exif.get_field(Tag::Orientation, In::PRIMARY) {
        match orientation.value.get_uint(0) {
            Some(v @ 1..=8) => {
                // Only orientations 1 through 8 are valid, so if it's outside that range we just error out:
                // https://web.archive.org/web/20200412005226/https://www.impulseadventure.com/photo/exif-orientation.html
                // We've already checked that it's within the right range, so we can cast and unwrap here.
                let orientation = Orientation::from_exif(v as u8).unwrap();
                image.apply_orientation(orientation);
                Ok(())
            },
            _ => Err(wm_err!("invalid Exif orientation value")),
        }
    } else {
        Ok(()) // no orientation value in Exif, nothing to do
    }
}
