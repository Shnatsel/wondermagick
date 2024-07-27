use exif::{In, Tag};
use image::DynamicImage;

use crate::{error::MagickError, wm_err, wm_try};

pub fn rotate_by_exif(image: &mut DynamicImage, raw_exif: Vec<u8>) -> Result<(), MagickError> {
    let reader = exif::Reader::new();
    let exif = wm_try!(reader.read_raw(raw_exif));
    // based on the Exif crate example: https://docs.rs/kamadak-exif/latest/exif/index.html#examples
    if let Some(orientation) = exif.get_field(Tag::Orientation, In::PRIMARY) {
        match orientation.value.get_uint(0) {
            Some(v @ 1..=8) => apply_exif_orientation(image, v as u8).map_err(|e| wm_err!("{}", e)),
            _ => Err(wm_err!("invalid Exif orientation value")),
        }
    } else {
        Ok(()) // no orientation value in Exif, nothing to do
    }
}

fn apply_exif_orientation(image: &mut DynamicImage, orientation: u8) -> Result<(), &'static str> {
    // An explanation of Exif orientation:
    // https://web.archive.org/web/20200412005226/https://www.impulseadventure.com/photo/exif-orientation.html
    match orientation {
        1 => Ok(()), // no transformations needed
        2 => Ok(image.fliph_in_place()),
        3 => Ok(image.rotate180_in_place()),
        4 => Ok(image.flipv_in_place()),
        5 => {
            let mut new_image = image.rotate90();
            new_image.fliph_in_place();
            *image = new_image;
            Ok(())
        }
        6 => Ok(*image = image.rotate90()),
        7 => {
            let mut new_image = image.rotate270();
            new_image.fliph_in_place();
            *image = new_image;
            Ok(())
        }
        8 => Ok(*image = image.rotate270()),
        0 | 9.. => return Err("invalid Exif orientation value"),
    }
}
