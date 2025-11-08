use std::io::Write;

use image::codecs::avif::AvifEncoder;
use image::ImageEncoder;

use crate::{error::MagickError, image::Image, plan::Modifiers, wm_try};

pub fn encode<W: Write>(
    image: &Image,
    writer: &mut W,
    modifiers: &Modifiers,
) -> Result<(), MagickError> {
    // TODO: quality conversion might not be bug-compatible with imagemagick
    let quality = modifiers.quality.map(|q| q as u8).unwrap_or(50);
    let mut encoder = AvifEncoder::new_with_speed_quality(writer, 4, quality);
    if let Some(icc) = image.icc.clone() {
        let _ = encoder.set_icc_profile(icc); // ignore UnsupportedError
    };
    // ravif already discards alpha channel automatically if all pixels are opaque,
    // so no need to explicitly convert on our end
    wm_try!(image.pixels.write_with_encoder(encoder));
    Ok(())
}
