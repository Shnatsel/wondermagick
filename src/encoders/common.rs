//! Helpers shared between all encoders

use crate::image::Image;
use image::ImageEncoder;

mod pixel_format_optimization;

pub(crate) use pixel_format_optimization::{
    is_opaque, optimize_pixel_format, optimize_pixel_format_and_precision,
};

pub fn write_icc_and_exif(encoder: &mut impl ImageEncoder, image: &Image) {
    if let Some(icc) = image.icc.clone() {
        let _ = encoder.set_icc_profile(icc); // ignore UnsupportedError
    };
    if let Some(exif) = image.exif.clone() {
        let _ = encoder.set_exif_metadata(exif); // ignore UnsupportedError
    };
}
