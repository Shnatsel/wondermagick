use std::{ffi::OsStr, fs::File, io::BufWriter, io::Write};

use image::ImageFormat;

use crate::{encoders, image::Image, plan::Modifiers, wm_try};

pub fn encode(
    image: &mut Image,
    file_path: &OsStr,
    format: Option<ImageFormat>,
    modifiers: &Modifiers,
) -> Result<(), crate::error::MagickError> {
    // This is a wrapper function that clears metadata if options like -strip are specified.
    //
    // Correctly stripping metadata when requested is a major privacy concern:
    // unstripped images may reveal the user's geographic location when phone cameras embed GPS coordinates.
    //
    // Therefore we do it here once and for all, without trusting any individual format handlers.
    let mut exif = None;
    let mut icc = None;
    if modifiers.strip.exif {
        exif = std::mem::take(&mut image.exif);
    }
    if modifiers.strip.icc {
        icc = std::mem::take(&mut image.icc);
    }

    // run the actual encoding function
    let result = encode_inner(image, file_path, format, modifiers);

    // restore the metadata to the image so that it could be used by subsequent operations like -auto-orient
    if exif.is_some() {
        image.exif = exif;
    }
    if icc.is_some() {
        image.exif = icc;
    }

    result
}

fn encode_inner(
    image: &Image,
    file_path: &OsStr,
    format: Option<ImageFormat>,
    modifiers: &Modifiers,
) -> Result<(), crate::error::MagickError> {
    // `File::create` automatically truncates (overwrites) the file if it exists.
    let file = wm_try!(File::create(file_path));
    // Wrap in BufWriter for performance
    let mut writer = BufWriter::new(file);

    // If format is unspecified, guess based on the output path;
    // if that fails, use the input format (like ImageMagick)
    let format = format
        .or_else(|| ImageFormat::from_path(file_path).ok())
        .unwrap_or(image.format);

    match format {
        // TODO: dedicated encoders for way more formats
        ImageFormat::Png => encoders::png::encode(image, &mut writer, modifiers)?,
        ImageFormat::Jpeg => encoders::jpeg::encode(image, &mut writer, modifiers)?,
        ImageFormat::WebP => encoders::webp::encode(image, &mut writer, modifiers)?,
        ImageFormat::Avif => encoders::avif::encode(image, &mut writer, modifiers)?,
        ImageFormat::Gif => encoders::gif::encode(image, &mut writer, modifiers)?,
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
