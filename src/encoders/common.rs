//! Helpers shared between all encoders

use crate::image::Image;
use image::ImageEncoder;

mod pixel_format_optimization;

pub(crate) use pixel_format_optimization::{
    optimize_pixel_format, optimize_pixel_format_and_precision, to_8bit_rgb_maybe_a,
};

pub fn write_metadata(encoder: &mut impl ImageEncoder, image: &Image) {
    if let Some(exif) = image.exif.clone() {
        let _ = encoder.set_exif_metadata(exif); // ignore UnsupportedError
    };
    if let Some(xmp) = image.xmp.clone() {
        let _ = encoder.set_xmp_metadata(xmp); // ignore UnsupportedError
    };
    if let Some(icc) = image.icc.clone() {
        let _ = encoder.set_icc_profile(icc); // ignore UnsupportedError
    };
}
