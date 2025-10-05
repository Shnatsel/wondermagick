use std::{ffi::OsStr, io::Write};

use image::ExtendedColorType;

use crate::{
    arg_parsers::IdentifyFormat, arg_parsers::Token, arg_parsers::Var, error::MagickError,
    image::Image, wm_try,
};

// https://imagemagick.org/script/command-line-options.php#identify
pub fn identify(image: &mut Image, format: Option<IdentifyFormat>) -> Result<(), MagickError> {
    // acquire a buffered writer to which we can make lots of small writes cheaply
    let mut stdout = std::io::stdout().lock();
    identify_impl(image, format, &mut stdout)
}

fn identify_impl(
    image: &Image,
    format: Option<IdentifyFormat>,
    writer: &mut impl Write,
) -> Result<(), MagickError> {
    // The default format, if none is specified, turns into something like
    // ~/imagename.jpg JPEG 1363x2048 1363x2048+0+0 8-bit sRGB 270336B 0.010u 0:00.013
    let template = match &format {
        Some(fmt) => &fmt.template,
        None => &vec![
            Token::Var(Var::ImageFilename),
            Token::Literal(" ".into()),
            Token::Var(Var::ImageFileFormat),
            Token::Literal(" ".into()),
            Token::Var(Var::CurrentImageWidthInPixels),
            Token::Literal("x".into()),
            Token::Var(Var::CurrentImageHeightInPixels),
            Token::Literal(" ".into()),
            Token::Var(Var::LayerCanvasPageGeometry),
            Token::Literal(" ".into()),
            Token::Var(Var::ImageDepth),
            Token::Literal("-bit ".into()),
            Token::Var(Var::Colorspace),
            Token::Newline,
            // TODO: file size in bytes
            // TODO: consumed user time identifying the image
            // TODO: elapsed time identifying the image
        ],
    };

    for token in template {
        match token {
            Token::Newline => wm_try!(write!(writer, "\n")),
            Token::Literal(text) => wm_try!(write!(writer, "{}", text)),
            Token::Var(Var::CurrentImageWidthInPixels | Var::PageCanvasWidth) => {
                wm_try!(write!(writer, "{}", image.pixels.width()));
            }
            Token::Var(Var::CurrentImageHeightInPixels | Var::PageCanvasHeight) => {
                wm_try!(write!(writer, "{}", image.pixels.height()));
            }
            Token::Var(Var::PageCanvasXOffset | Var::PageCanvasYOffset) => {
                // TODO: actually read and report these offsets
                wm_try!(write!(writer, "+{}", 0));
            }
            Token::Var(Var::ImageFileFormat) => {
                if let Some(format) = image.format.map(|f| f.extensions_str()[0].to_uppercase()) {
                    wm_try!(write!(writer, "{}", format));
                }
            }
            Token::Var(Var::ImageFilename | Var::MagickFilename) => {
                write_filename(&image.properties.filename, writer)?;
            }
            Token::Var(Var::OriginalImageSize) => {
                wm_try!(write!(
                    writer,
                    "{}x{}",
                    image.pixels.width(),
                    image.pixels.height()
                ));
            }
            Token::Var(Var::LayerCanvasPageGeometry) => {
                let dimensions = format!("{}x{}", image.pixels.width(), image.pixels.height());
                // TODO: actually read and report these offsets
                wm_try!(write!(writer, "{}+0+0", dimensions));
            }
            Token::Var(Var::ImageDepth) => {
                let color_type = image.properties.color_type;
                let bits_per_channel =
                    color_type.bits_per_pixel() / color_type.channel_count() as u16;
                wm_try!(write!(writer, "{}", bits_per_channel));
            }
            Token::Var(Var::Colorspace) => {
                let color_type = image.properties.color_type;
                if let Some(colorspace) = get_colorspace(color_type) {
                    wm_try!(write!(writer, "{}", colorspace));
                }
            }
        }
    }

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
        wm_try!(write!(writer, "{}", filename.to_string_lossy()));
    }
    Ok(())
}

fn get_colorspace(color_type: ExtendedColorType) -> Option<String> {
    // TODO: distingush between sRGB and RGB, Gray and LinearGray.
    //
    // List of recognized color spaces:
    // https://imagemagick.org/script/command-line-options.php#colorspace
    use ExtendedColorType::*;
    let string = match color_type {
        A8 => "Transparent",
        L1 | L2 | L4 | L8 | L16 => "Gray",
        La1 | La2 | La4 | La8 | La16 => todo!(),
        Rgba1 | Rgba2 | Rgba4 | Rgba8 | Rgba16 => "sRGB",
        Rgb1 | Rgb2 | Rgb4 | Rgb8 | Rgb16 => "sRGB",
        Bgr8 | Bgra8 => "sRGB",
        Rgb32F | Rgba32F => "sRGB",
        Cmyk8 => "CMYK",
        Unknown(_) => return None,
        _ => return None,
    };
    Some(string.to_owned())
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
        let width = width.get() as u32;
        let height = height.get() as u32;
        let image = DynamicImage::new_rgba8(width, height);
        let mut output = Vec::new();
        identify_impl(
            &mut Image {
                format: Some(image::ImageFormat::Png),
                exif: None,
                icc: None,
                pixels: image,
                properties: InputProperties {
                    filename: "/some/path/test.png".into(),
                    color_type: ExtendedColorType::Rgb16,
                },
            },
            None,
            &mut output,
        )
        .unwrap();
        assert_eq!(
            String::try_from(output).unwrap(),
            format!("/some/path/test.png PNG {width}x{height} {width}x{height}+0+0 16-bit sRGB\n")
        );
    }

    #[test]
    fn test_identify_with_format_template_vars() {
        let mut output = Vec::new();
        identify_impl(
            &mut Image {
                format: None,
                exif: None,
                icc: None,
                pixels: DynamicImage::new_rgba8(123, 42),
                properties: InputProperties {
                    filename: "irrelevant.jpg".into(),
                    color_type: ExtendedColorType::Cmyk8,
                },
            },
            Some(IdentifyFormat {
                template: vec![
                    Token::Var(Var::CurrentImageWidthInPixels),
                    Token::Literal("   ".into()),
                    Token::Var(Var::CurrentImageHeightInPixels),
                    Token::Newline,
                ],
            }),
            &mut output,
        )
        .unwrap();
        assert_eq!(String::try_from(output).unwrap(), "123   42\n");
    }

    #[test]
    fn test_identify_with_format_template_literal() {
        let mut output = Vec::new();
        identify_impl(
            &mut Image {
                format: None,
                exif: None,
                icc: None,
                pixels: DynamicImage::new_rgba8(1, 1),
                properties: InputProperties {
                    filename: "irrelevant.jpg".into(),
                    color_type: ExtendedColorType::Cmyk8,
                },
            },
            Some(IdentifyFormat {
                template: vec![Token::Literal("text".into())],
            }),
            &mut output,
        )
        .unwrap();
        assert_eq!(String::try_from(output).unwrap(), "text");
    }
}
