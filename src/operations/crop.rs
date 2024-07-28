use crate::arg_parsers::LoadCropGeometry;

pub fn crop_on_load(
    image: &mut image::DynamicImage,
    geom: &LoadCropGeometry,
) -> Result<(), crate::error::MagickError> {
    // Sadly this doesn't check bounds right now, so we can get panics later on because of wrong crop parameters:
    // https://github.com/image-rs/image/issues/2296
    // TODO: change this in `image` because I don't want to emulate this on the client side
    // and pretend the problem doesn't exist for anyone else
    let cropped = image.crop_imm(geom.xoffset, geom.yoffset, geom.width, geom.height);
    *image = cropped;
    Ok(())
}
