use crate::{arg_parsers::GrayscaleMethod, error::MagickError, image, wm_err};

pub fn grayscale(image: &mut image::Image, method: &GrayscaleMethod) -> Result<(), MagickError> {
    match method {
        GrayscaleMethod::Rec709Luma => {
            // image-rs appears to be using something like Rec. 709 by default for grayscale
            // conversion of most image types.
            // https://github.com/image-rs/image/blob/2e121abff5f87028e85bf8f26a95f36f7b6182ac/src/images/buffer.rs#L1577
            // https://github.com/image-rs/image/issues/598
            image.pixels = image.pixels.grayscale();
            Ok(())
        }
        _ => Err(wm_err!("grayscale method not implemented")),
    }
}
