use std::{ffi::OsStr, fs::File, io::BufWriter, io::Write};

use image::ImageFormat;

use crate::{error::MagickError, image::Image, plan::Modifiers, wm_try};

use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::ImageEncoder;

pub fn encode(
    image: &Image,
    file_path: &OsStr,
    format: Option<ImageFormat>,
    modifiers: &Modifiers,
) -> Result<(), crate::error::MagickError> {
    // `File::create` automatically truncates (overwrites) the file if it exists.
    let file = wm_try!(File::create(file_path));
    // Wrap in BufWriter for performance
    let mut writer = BufWriter::new(file);

    let format = if let Some(format) = format {
        format
    } else {
        // TODO: instead of rejecting unknown format, reuse the input format as imagemagick does
        wm_try!(ImageFormat::from_path(file_path))
    };

    match format {
        // TODO: dedicated encoders for way more formats
        ImageFormat::Png => encode_png(image, &mut writer, modifiers)?,
        ImageFormat::Jpeg => encode_jpeg(image, &mut writer, modifiers)?,
        // TODO: handle format conversions such as RGBA -> RGB, 16-bit to 8-bit, etc.
        // Blocked on https://github.com/image-rs/image/issues/2498
        _ => wm_try!(image.pixels.write_to(&mut writer, format)),
    }

    // Flush the buffers to write everything to disk.
    // The buffers will be flushed automatically when the writer goes out of scope,
    // but that will not report any errors. This handles errors.
    wm_try!(writer.flush());

    Ok(())
}

pub fn encode_jpeg<W: Write>(
    image: &Image,
    writer: &mut W,
    modifiers: &Modifiers,
) -> Result<(), MagickError> {
    // imagemagick estimates the quality of the input JPEG somehow according to
    // https://www.imagemagick.org/script/command-line-options.php#quality
    // but we don't do that yet
    let mut encoder = JpegEncoder::new_with_quality(writer, modifiers.quality.unwrap_or(92));
    if let Some(icc) = image.icc.clone() {
        let _ = encoder.set_icc_profile(icc); // ignore UnsupportedError
    };
    Ok(wm_try!(image.pixels.write_with_encoder(encoder)))
}

pub fn encode_png<W: Write>(
    image: &Image,
    writer: &mut W,
    modifiers: &Modifiers,
) -> Result<(), MagickError> {
    let (compression, filter) = quality_to_png_parameters(modifiers.quality);
    let mut encoder = PngEncoder::new_with_quality(writer, compression, filter);
    if let Some(icc) = image.icc.clone() {
        let _ = encoder.set_icc_profile(icc); // ignore UnsupportedError
    };
    Ok(wm_try!(image.pixels.write_with_encoder(encoder)))
}

// for documentation on conversion of quality to encoding parameters see
// https://www.imagemagick.org/script/command-line-options.php#quality
fn quality_to_png_parameters(quality: Option<u8>) -> (CompressionType, FilterType) {
    if let Some(quality) = quality {
        // TODO: correct quality mapping is blocked on upstream issue:
        // https://github.com/image-rs/image/issues/2495
        let compression = match quality / 10 {
            0..=2 => CompressionType::Fast,
            3..=7 => CompressionType::Default,
            8.. => CompressionType::Best,
        };
        let filter = match quality % 10 {
            0 => FilterType::NoFilter,
            1 => FilterType::Sub,
            2 => FilterType::Up,
            3 => FilterType::Avg,
            4 => FilterType::Paeth,
            // 7 is documented as MNG-only, in practice maps to 5 or 6?
            5..=7 => FilterType::Adaptive,
            // filters 8 and 9 override compression level selection
            8 => return (CompressionType::Fast, FilterType::Adaptive),
            // imagemagick uses filter=None here, but our Fast mode needs filtering
            // to deliver reasonable compression, so use the fastest filter instead
            9 => return (CompressionType::Fast, FilterType::Up),
            10.. => unreachable!(),
        };

        if filter == FilterType::NoFilter && compression == CompressionType::Fast {
            // CompressionType::Fast needs filtering for a reasonable compression ratio.
            // When using it, use the fastest filter instead of no filter at all.
            (CompressionType::Fast, FilterType::Up)
        } else {
            (compression, filter)
        }
    } else {
        (CompressionType::Default, FilterType::Adaptive)
    }
}
