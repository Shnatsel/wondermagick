use std::io::Write;

use crate::{error::MagickError, image::Image, plan::Modifiers, wm_err, wm_try};
use image::DynamicImage;
use oxideav_core::{CodecId, CodecParameters, Frame, PixelFormat, VideoFrame, VideoPlane};
use oxideav_webp::{encoder, encoder_vp8, CODEC_ID_VP8, CODEC_ID_VP8L};

pub fn encode<W: Write>(
    image: &Image,
    writer: &mut W,
    modifiers: &Modifiers,
) -> Result<(), MagickError> {
    let (pixel_format, frame) = webp_video_frame(&image.pixels);

    // imagemagick signals that the image should be lossless with quality=100
    let lossless = modifiers.quality == Some(100.0);
    // default quality is not documented, was determined experimentally
    let quality = modifiers.quality.unwrap_or(75.0) as f32;

    let mut params = CodecParameters::video(CodecId::new(if lossless {
        CODEC_ID_VP8L
    } else {
        CODEC_ID_VP8
    }));
    params.width = Some(image.pixels.width());
    params.height = Some(image.pixels.height());
    params.pixel_format = Some(pixel_format);

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

fn webp_video_frame(image: &DynamicImage) -> (PixelFormat, Frame) {
    match image {
        DynamicImage::ImageRgb8(pixels) => video_frame(
            PixelFormat::Rgb24,
            pixels.width() as usize * 3,
            pixels.as_raw().clone(),
        ),
        DynamicImage::ImageRgba8(pixels) => video_frame(
            PixelFormat::Rgba,
            pixels.width() as usize * 4,
            pixels.as_raw().clone(),
        ),
        image if !image.has_alpha() => {
            let pixels = image.to_rgb8();
            video_frame(
                PixelFormat::Rgb24,
                pixels.width() as usize * 3,
                pixels.into_raw(),
            )
        }
        image => {
            let pixels = image.to_rgba8();
            video_frame(
                PixelFormat::Rgba,
                pixels.width() as usize * 4,
                pixels.into_raw(),
            )
        }
    }
}

fn video_frame(pixel_format: PixelFormat, stride: usize, data: Vec<u8>) -> (PixelFormat, Frame) {
    (
        pixel_format,
        Frame::Video(VideoFrame {
            pts: Some(0),
            planes: vec![VideoPlane { stride, data }],
        }),
    )
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use image::{DynamicImage, ExtendedColorType, ImageFormat, RgbImage, RgbaImage};

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

    fn test_rgb_image() -> Image {
        let pixels =
            RgbImage::from_raw(2, 2, vec![255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255]).unwrap();
        Image {
            format: Some(ImageFormat::Png),
            exif: None,
            icc: None,
            pixels: DynamicImage::ImageRgb8(pixels),
            properties: InputProperties {
                filename: OsString::from("test-rgb.png"),
                color_type: ExtendedColorType::Rgb8,
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
    fn rgb8_input_uses_rgb24_frame() {
        let image = test_rgb_image();
        let (pixel_format, frame) = webp_video_frame(&image.pixels);

        assert_eq!(pixel_format, PixelFormat::Rgb24);
        let Frame::Video(frame) = frame else {
            unreachable!();
        };
        assert_eq!(frame.planes[0].stride, 6);
        assert_eq!(frame.planes[0].data.len(), 12);
    }

    #[test]
    fn encodes_lossy_rgb_webp_without_alpha_chunk() {
        let mut output = Vec::new();
        encode(&test_rgb_image(), &mut output, &Modifiers::default()).unwrap();

        assert_eq!(&output[0..4], b"RIFF");
        assert_eq!(&output[8..12], b"WEBP");
        assert_eq!(&output[12..16], b"VP8 ");
        let decoded = oxideav_webp::decode_webp(&output).unwrap();
        assert_eq!((decoded.width, decoded.height), (2, 2));
    }

    #[test]
    fn quality_100_encodes_lossless_rgb_webp() {
        let mut output = Vec::new();
        let modifiers = Modifiers {
            quality: Some(100.0),
            ..Modifiers::default()
        };
        encode(&test_rgb_image(), &mut output, &modifiers).unwrap();

        assert_eq!(&output[0..4], b"RIFF");
        assert_eq!(&output[8..12], b"WEBP");
        assert_eq!(&output[12..16], b"VP8L");
        let decoded = oxideav_webp::decode_webp(&output).unwrap();
        assert_eq!((decoded.width, decoded.height), (2, 2));
        assert_eq!(
            decoded.frames[0].rgba,
            vec![255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,]
        );
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
