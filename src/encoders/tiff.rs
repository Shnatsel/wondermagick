use std::io::{Seek, Write};

use image::codecs::tiff::TiffEncoder;

use crate::encoders::common::write_icc_and_exif;
use crate::{error::MagickError, image::Image, plan::Modifiers, wm_try};

pub fn encode<W: Write + Seek>(
    image: &Image,
    writer: &mut W,
    _modifiers: &Modifiers,
) -> Result<(), MagickError> {
    let mut encoder = TiffEncoder::new(writer);
    write_icc_and_exif(&mut encoder, image);
    wm_try!(image.pixels.write_with_encoder(encoder));
    Ok(())
}
