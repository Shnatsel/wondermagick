use std::io::Write;

use image::codecs::gif::GifEncoder;

use crate::{error::MagickError, image::Image, plan::Modifiers, wm_try};

pub fn encode<W: Write>(
    image: &Image,
    writer: &mut W,
    _modifiers: &Modifiers,
) -> Result<(), MagickError> {
    // speed needs to be set manually due to https://github.com/image-rs/image/issues/2506
    let encoder = GifEncoder::new_with_speed(writer, 10);
    wm_try!(image.pixels.write_with_encoder(encoder));
    Ok(())
}
