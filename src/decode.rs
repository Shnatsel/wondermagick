use std::ffi::OsStr;

use image::{DynamicImage, ImageDecoder, ImageFormat, ImageReader};

use crate::{error::MagickError, image::Image, wm_try};

/// If the format has not been explicitly specified, guesses the format based on file contents.
pub fn decode(file: &OsStr, format: Option<ImageFormat>) -> Result<Image, MagickError> {
    let mut reader = wm_try!(ImageReader::open(file));
    let format = match format {
        Some(format) => {
            reader.set_format(format);
            format
        }
        None => {
            reader = wm_try!(reader.with_guessed_format());
            reader.format().unwrap()
        }
    };
    let mut decoder = wm_try!(reader.into_decoder());
    let exif = decoder.exif_metadata().unwrap_or(None);
    let icc = decoder.icc_profile().unwrap_or(None);
    let pixels = wm_try!(DynamicImage::from_decoder(decoder));
    Ok(Image {
        format,
        exif,
        icc,
        pixels,
    })
}

// You know what would be a cool optimization for decoding process?
// Using the image thumbnail from the Exif metadata, when it's present.
// Many JPEG images have it; other formats can contain it as well.
// That way you don't even have to decode the full-size image!
// You can dump it from a JPEG that has one like this:
// $ exiftool -b -ThumbnailImage image.jpg > ~/thumb.jpg
// It saves so much time! Every competent thumbnailer uses it!
//
// But imagemagick is not a competent thumbnailer.
// Imagemagick not only ignores it, it does something far worse:
// It keeps the old thumbnail even after modifying the image.
// Making the embedded thumbnail WRONG.
//
// Not only does it not make use of the thumbnail that someone
// has already spent network bandwidth and storage on,
// but it also breaks any other program making use of it,
// by keeping the old thumbnail around
// even when it doesn't match the image anymore.
//
// This is completely wrong behavior no matter how you look at it.
// But that's what imagemagick does and what we must do as well.
// Until we decide enough is enough and some bugs aren't worth
// being compatible with, but that day is not today.
