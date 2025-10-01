use crate::{error::MagickError, image::Image};
use std::ffi::OsStr;

// https://imagemagick.org/script/command-line-options.php#identify
pub fn identify(filename: &OsStr, image: &mut Image) -> Result<(), MagickError> {
    println!("{}", identify_impl(filename, image));
    Ok(())
}

fn identify_impl(filename: &OsStr, image: &Image) -> String {
    let parts: Vec<String> = vec![
        filename.to_str().map(str::to_owned),
        image.format.map(|f| f.extensions_str()[0].to_uppercase()),
        Some(format!(
            "{}x{}",
            image.pixels.width(),
            image.pixels.height()
        )),
    ]
    .into_iter()
    .flatten()
    .collect();

    parts.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::DynamicImage;

    use quickcheck_macros::quickcheck;
    use std::num::NonZeroU8;

    #[quickcheck]
    // u8::MAX * u8::MAX is a large enough space for
    // quickcheck to explore and verify and still runs quickly
    fn test_identify(width: NonZeroU8, height: NonZeroU8) {
        let image = DynamicImage::new_rgba8(width.get() as u32, height.get() as u32);
        let filename = OsStr::new("/some/path/test.png");
        let info = identify_impl(
            &filename,
            &mut Image {
                format: Some(image::ImageFormat::Png),
                exif: None,
                icc: None,
                pixels: image,
            },
        );
        assert_eq!(
            info,
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
        let filename = OsStr::new("image_without_format.jpg");
        let info = identify_impl(
            &filename,
            &mut Image {
                format: None,
                exif: None,
                icc: None,
                pixels: image,
            },
        );
        assert_eq!(info, "image_without_format.jpg 1x1");
    }
}
