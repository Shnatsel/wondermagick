use std::{ffi::OsStr, io::Write};

use crate::{error::MagickError, image::Image, wm_try};

// https://imagemagick.org/script/command-line-options.php#identify
pub fn identify(image: &mut Image) -> Result<(), MagickError> {
    // acquire a buffered writer to which we can make lots of small writes cheaply
    let mut stdout = std::io::stdout().lock();
    identify_impl(image, &mut stdout)
}

fn identify_impl(image: &Image, writer: &mut impl Write) -> Result<(), MagickError> {
    write_filename(&image.properties.filename, writer)?;

    let format = image.format.map(|f| f.extensions_str()[0].to_uppercase());
    let dimensions = Some(format!(
        "{}x{}",
        image.pixels.width(),
        image.pixels.height()
    ));

    let parts: Vec<String> = vec![format, dimensions].into_iter().flatten().collect();

    wm_try!(writeln!(writer, "{}", parts.join(" ")));
    Ok(())
}

fn write_filename(filename: &OsStr, writer: &mut impl Write) -> Result<(), MagickError> {
    #[cfg(unix)]
    {
        // On Unix, OsStr is just a &[u8], and filenames are allowed to have non-UTF-8 bytes.
        // Imagemagick outputs those bytes verbatim, and this replicates that behavior.
        use std::os::unix::ffi::OsStrExt;
        wm_try!(writer.write_all(filename.as_bytes()));
    }
    #[cfg(not(unix))]
    {
        // Windows stores filenames as UTF-16 that isn't required to be valid.
        // That isn't printable verbatim to Windows console, so we debug-print them with escaping.
        // TODO: figure out what, if anything, imagemagick does on Windows for non-UTF-16 filenames and replicate that.
        wm_try!(write!(writer, "{:?}", filename));
    }
    // write the space separator after the filename
    wm_try!(write!(writer, " "));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::image::InputProperties;
    use image::{DynamicImage, ExtendedColorType};

    use quickcheck_macros::quickcheck;
    use std::num::NonZeroU8;

    #[quickcheck]
    // u8::MAX * u8::MAX is a large enough space for
    // quickcheck to explore and verify and still runs quickly
    fn test_identify(width: NonZeroU8, height: NonZeroU8) {
        let image = DynamicImage::new_rgba8(width.get() as u32, height.get() as u32);
        let mut output = Vec::new();
        identify_impl(
            &mut Image {
                format: Some(image::ImageFormat::Png),
                exif: None,
                icc: None,
                pixels: image,
                properties: InputProperties {
                    filename: "/some/path/test.png".into(),
                    color_type: ExtendedColorType::A8,
                },
            },
            &mut output,
        )
        .unwrap();
        assert_eq!(
            String::try_from(output).unwrap(),
            format!(
                "/some/path/test.png PNG {}x{}",
                width.get() as u32,
                height.get() as u32
            )
        );
    }

    #[test]
    fn test_identify_without_format() {
        let image = DynamicImage::new_rgba8(1, 1);
        let mut output = Vec::new();
        identify_impl(
            &mut Image {
                format: None,
                exif: None,
                icc: None,
                pixels: image,
                properties: InputProperties {
                    filename: "image_without_format.jpg".into(),
                    color_type: ExtendedColorType::A8,
                },
            },
            &mut output,
        )
        .unwrap();
        assert_eq!(
            String::try_from(output).unwrap(),
            "image_without_format.jpg 1x1"
        );
    }
}
