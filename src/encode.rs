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

    let format = if let Some(format) = format {
        format
    } else {
        // TODO: instead of rejecting unknown format, reuse the input format as imagemagick does
        wm_try!(ImageFormat::from_path(file_path))
    };

    match format {
        // TODO: dedicated encoders for way more formats
        ImageFormat::Png => encoders::png::encode(image, &mut writer, modifiers)?,
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
