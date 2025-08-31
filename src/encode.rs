use std::{ffi::OsStr, fs::File, io::BufWriter, io::Write};

use image::ImageFormat;

use crate::{encoders, image::Image, plan::Modifiers, wm_try};

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
