use std::io::Write;

use crate::encoders::common::to_8bit_rgb_maybe_a;
use crate::{error::MagickError, image::Image, plan::Modifiers, wm_err, wm_try};
use image::DynamicImage;
use webpx::{EncoderConfig, Unstoppable};

pub fn encode<W: Write>(
    image: &Image,
    writer: &mut W,
    modifiers: &Modifiers,
) -> Result<(), MagickError> {
    // Convert the image to Rgb(a)8, because those are the only formats the encoder supports
    let pixels = to_8bit_rgb_maybe_a(&image.pixels);
    // imagemagick signals that the image should be lossless with quality=100
    let lossless = modifiers.quality == Some(100.0);
    // default quality is not documented, was determined experimentally
    let quality = modifiers.quality.unwrap_or(75.0) as f32;

    let mut config = EncoderConfig::new().quality(quality).lossless(lossless);
    if let Some(icc) = image.icc.clone() {
        config = config.icc_profile(icc);
    }
    if let Some(exif) = image.exif.clone() {
        config = config.exif(exif);
    }
    if let Some(xmp) = image.xmp.clone() {
        config = config.xmp(xmp);
    }

    let webp = match pixels.as_ref() {
        DynamicImage::ImageRgb8(pixels) => config.encode_rgb(
            pixels.as_raw(),
            pixels.width(),
            pixels.height(),
            Unstoppable,
        ),
        DynamicImage::ImageRgba8(pixels) => config.encode_rgba(
            pixels.as_raw(),
            pixels.width(),
            pixels.height(),
            Unstoppable,
        ),
        _ => unreachable!("to_8bit_rgb_maybe_a only returns Rgb8 or Rgba8"),
    }
    .map_err(|e| wm_err!("WebP encoding failed: {e:?}"))?;

    wm_try!(writer.write_all(&webp));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::image::InputProperties;
    use image::{ExtendedColorType, ImageFormat};

    fn image_with_metadata() -> Image {
        Image {
            format: Some(ImageFormat::WebP),
            exif: Some(vec![1, 2, 3]),
            xmp: Some(
                br#"<x:xmpmeta xmlns:x="adobe:ns:meta/"><rdf:RDF></rdf:RDF></x:xmpmeta>"#.to_vec(),
            ),
            icc: Some(vec![4, 5, 6]),
            pixels: DynamicImage::new_rgba8(2, 2),
            properties: InputProperties {
                filename: "input.webp".into(),
                color_type: ExtendedColorType::Rgba8,
            },
        }
    }

    #[test]
    fn embeds_metadata() {
        let image = image_with_metadata();
        let mut output = Vec::new();

        encode(&image, &mut output, &Modifiers::default()).unwrap();

        assert_eq!(webpx::get_exif(&output).unwrap(), image.exif);
        assert_eq!(webpx::get_xmp(&output).unwrap(), image.xmp);
        assert_eq!(webpx::get_icc_profile(&output).unwrap(), image.icc);
    }
}
