use std::{
    ffi::OsStr,
    fs::File,
    io::{BufWriter, Seek, Write},
    path::Path,
};

use image::ImageFormat;

use crate::{
    arg_parsers::Location, encoders, error::MagickError, image::Image, plan::Modifiers, wm_err,
    wm_try,
};

pub fn encode(
    image: &mut Image,
    location: &Location,
    format: Option<FileFormat>,
    modifiers: &Modifiers,
) -> Result<(), MagickError> {
    let format = match format {
        // no-op, return immediately
        Some(FileFormat::DoNotEncode) => return Ok(()),
        Some(FileFormat::Format(fmt)) => Some(fmt),
        None => None,
    };

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
    let result = encode_inner(image, location, format, modifiers);

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
    location: &Location,
    format: Option<ImageFormat>,
    modifiers: &Modifiers,
) -> Result<(), MagickError> {
    let format = choose_encoding_format(image, location, format)?;

    let file = match location {
        // `File::create` automatically truncates (overwrites) the file if it exists.
        Location::Path(path) => File::create(path)
            .map_err(|error| wm_err!("unable to open image '{}': {error}", path.display()))?,
        // Some of the encoders require Seek, which Stdout doesn't implement.
        // We write to a temporary file and then print out the content at the end.
        Location::Stdio => wm_try!(tempfile::tempfile()),
    };
    // Wrap in BufWriter for performance
    let mut writer = BufWriter::new(file);

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

    match location {
        Location::Path(_) => {
            // Flush the buffers to write everything to disk.
            // The buffers will be flushed automatically when the writer goes out of scope,
            // but that will not report any errors. This handles errors.
            wm_try!(writer.flush());
        }
        Location::Stdio => {
            // Copy from the temporary file to stdout while handling errors
            let mut file = wm_try!(writer.into_inner());
            wm_try!(file.seek(std::io::SeekFrom::Start(0)));
            let mut stdout = std::io::stdout().lock();
            wm_try!(std::io::copy(&mut file, &mut stdout));
            wm_try!(stdout.flush());
        }
    }

    Ok(())
}

fn choose_encoding_format(
    image: &Image,
    location: &Location,
    explicitly_specified: Option<ImageFormat>,
) -> Result<ImageFormat, MagickError> {
    if let Some(format) = explicitly_specified {
        return Ok(format);
    }
    // if format was not explicitly specified, guess based on the output path
    if let Location::Path(path) = location {
        if let Ok(format) = ImageFormat::from_path(path) {
            return Ok(format);
        }
    }
    // if that fails, use the input format (like ImageMagick)
    if let Some(format) = image.format {
        return Ok(format);
    }
    // otherwise error
    let extension = match location {
        // fallback to empty string matches imagemagick
        Location::Path(path) => Path::new(path).extension().unwrap_or(OsStr::new("")),
        Location::Stdio => OsStr::new(""),
    };
    Err(wm_err!(
        "no encode delegate for this image format `{}'",
        extension.to_ascii_uppercase().display()
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileFormat {
    Format(ImageFormat),
    /// Encoding operation is present but is a no-op. On the CLI this is "null:" passed as filename.
    DoNotEncode,
}

impl FileFormat {
    /// Creates a format from the explicit specifier that precedes the filename,
    /// e.g. `png:my-file` or `null:`
    pub fn from_prefix(prefix: &str) -> Option<Self> {
        let lowercase_prefix = prefix.to_ascii_lowercase();
        let format = if lowercase_prefix == "null" {
            Self::DoNotEncode
        } else {
            Self::Format(ImageFormat::from_extension(lowercase_prefix)?)
        };
        Some(format)
    }
}
