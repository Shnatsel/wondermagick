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

    #[quickcheck]
    // u8::MAX * u8::MAX is a large enough space for
    // quickcheck to explore and verify and still runs quickly
    fn test_identify(width: u8, height: u8) -> bool {
        let image = DynamicImage::new_rgba8((width as u32) + 1, (height as u32) + 1);
        let info = identify(&mut Image {
            format: image::ImageFormat::Png,
            exif: None,
            icc: None,
            pixels: image,
        })
        .unwrap();
        info == format!("{}x{}", width, height)
    }
}
