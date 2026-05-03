use std::io::Write;

use crate::{error::MagickError, image::Image, plan::Modifiers, wm_err, wm_try};
use oxideav_core::{CodecId, CodecParameters, Frame, PixelFormat, VideoFrame, VideoPlane};
use oxideav_webp::{encoder, encoder_vp8, CODEC_ID_VP8, CODEC_ID_VP8L};

pub fn encode<W: Write>(
    image: &Image,
    writer: &mut W,
    modifiers: &Modifiers,
) -> Result<(), MagickError> {
    // oxideav-webp currently takes only RGBA frames on this path.
    // additionally, oxideav-core VideoFrame needs to own its buffer,
    // so we have to make a copy here since we're only handed an &Image
    let rgba = image.pixels.to_rgba8();
    let width = rgba.width();
    let height = rgba.height();
    let frame = Frame::Video(VideoFrame {
        pts: Some(0),
        planes: vec![VideoPlane {
            stride: width as usize * 4,
            data: rgba.into_raw(),
        }],
    });

    // imagemagick signals that the image should be lossless with quality=100
    let lossless = modifiers.quality == Some(100.0);
    // default quality is not documented, was determined experimentally
    let quality = modifiers.quality.unwrap_or(75.0) as f32;

    let mut params = CodecParameters::video(CodecId::new(if lossless {
        CODEC_ID_VP8L
    } else {
        CODEC_ID_VP8
    }));
    params.width = Some(width);
    params.height = Some(height);
    params.pixel_format = Some(PixelFormat::Rgba);

    let mut encoder = if lossless {
        encoder::make_encoder(&params).map_err(|e| wm_err!("WebP encoding failed: {e}"))?
    } else {
        encoder_vp8::make_encoder_with_quality(&params, quality)
            .map_err(|e| wm_err!("WebP encoding failed: {e}"))?
    };

    encoder
        .send_frame(&frame)
        .map_err(|e| wm_err!("WebP encoding failed: {e}"))?;
    encoder
        .flush()
        .map_err(|e| wm_err!("WebP encoding failed: {e}"))?;
    let webp = encoder
        .receive_packet()
        .map_err(|e| wm_err!("WebP encoding failed: {e}"))?;

    // TODO: oxideav-webp can write ICC, EXIF, and XMP metadata via WebpMetadata.
    wm_try!(writer.write_all(&webp.data));
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use image::{DynamicImage, ExtendedColorType, ImageFormat, RgbaImage};

    use super::*;
    use crate::image::InputProperties;

    fn test_image() -> Image {
        let pixels = RgbaImage::from_raw(
            2,
            2,
            vec![
                255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 128, 255, 255, 255, 255,
            ],
        )
        .unwrap();
        Image {
            format: Some(ImageFormat::Png),
            exif: None,
            icc: None,
            pixels: DynamicImage::ImageRgba8(pixels),
            properties: InputProperties {
                filename: OsString::from("test.png"),
                color_type: ExtendedColorType::Rgba8,
            },
        }
    }

    #[test]
    fn encodes_lossy_webp() {
        let mut output = Vec::new();
        encode(&test_image(), &mut output, &Modifiers::default()).unwrap();

        assert_eq!(&output[0..4], b"RIFF");
        assert_eq!(&output[8..12], b"WEBP");
        let decoded = oxideav_webp::decode_webp(&output).unwrap();
        assert_eq!((decoded.width, decoded.height), (2, 2));
    }

    #[test]
    fn quality_100_encodes_lossless_webp() {
        let mut output = Vec::new();
        let modifiers = Modifiers {
            quality: Some(100.0),
            ..Modifiers::default()
        };
        encode(&test_image(), &mut output, &modifiers).unwrap();

        assert_eq!(&output[0..4], b"RIFF");
        assert_eq!(&output[8..12], b"WEBP");
        let decoded = oxideav_webp::decode_webp(&output).unwrap();
        assert_eq!((decoded.width, decoded.height), (2, 2));
    }
}
