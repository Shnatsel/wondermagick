use std::{
    ffi::OsStr,
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use image::ImageFormat;

use crate::{encoders, error::MagickError, image::Image, plan::Modifiers, wm_err, wm_try};

pub fn encode(
    image: &mut Image,
    file_path: &OsStr,
    format: Option<ImageFormat>,
    modifiers: &Modifiers,
) -> Result<(), MagickError> {
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
) -> Result<(), MagickError> {
    // `File::create` automatically truncates (overwrites) the file if it exists.
    let file = wm_try!(File::create(file_path));
    // Wrap in BufWriter for performance
    let mut writer = BufWriter::new(file);

    let format = choose_encoding_format(image, file_path, format)?;

    match format {
        // TODO: dedicated encoders for all other formats that have quality settings
        #[cfg(feature = "png")]
        ImageFormat::Png => encoders::png::encode(image, &mut writer, modifiers)?,
        #[cfg(feature = "jpeg")]
        ImageFormat::Jpeg => encoders::jpeg::encode(image, &mut writer, modifiers)?,
        #[cfg(feature = "webp")]
        ImageFormat::WebP => encoders::webp::encode(image, &mut writer, modifiers)?,
        #[cfg(feature = "avif")]
        ImageFormat::Avif => encoders::avif::encode(image, &mut writer, modifiers)?,
        #[cfg(feature = "gif")]
        ImageFormat::Gif => encoders::gif::encode(image, &mut writer, modifiers)?,
        // TODO: set the metadata generically on all the abstract formats.
        // Requires https://github.com/image-rs/image/pull/2554 or equivalent.
        _ => wm_try!(image.pixels.write_to(&mut writer, format)),
    }

    // Flush the buffers to write everything to disk.
    // The buffers will be flushed automatically when the writer goes out of scope,
    // but that will not report any errors. This handles errors.
    wm_try!(writer.flush());

    Ok(())
}

fn choose_encoding_format(
    image: &Image,
    file_path: &OsStr,
    explicitly_specified: Option<ImageFormat>,
) -> Result<ImageFormat, MagickError> {
    if let Some(format) = explicitly_specified {
        Ok(format)
    // if format was not explicitly specified, guess based on the output path
    } else if let Ok(format) = ImageFormat::from_path(file_path) {
        Ok(format)
    // if that fails, use the input format (like ImageMagick)
    } else if let Some(format) = image.format {
        Ok(format)
    } else {
        // fallback to emptry string matches imagemagick
        let extension = Path::new(file_path).extension().unwrap_or(OsStr::new(""));
        Err(wm_err!(
            "no decode delegate for this image format `{}'",
            extension.display()
        ))
    }
}
