use crate::{error::MagickError, image::Image};

pub fn identify(image: &mut Image) -> Result<String, MagickError> {
    Ok(format!(
        "{}x{}",
        image.pixels.width(),
        image.pixels.height()
    ))
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
        let info = identify(&mut Image {
            format: Some(image::ImageFormat::Png),
            exif: None,
            icc: None,
            pixels: image,
        })
        .unwrap();
        assert_eq!(
            info,
            format!("{}x{}", width.get() as u32, height.get() as u32)
        );
    }
}
