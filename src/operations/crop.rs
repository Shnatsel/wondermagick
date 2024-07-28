use crate::{arg_parsers::LoadCropGeometry, wm_try};

pub fn crop_on_load(
    image: &mut image::DynamicImage,
    geom: &LoadCropGeometry,
) -> Result<(), crate::error::MagickError> {
    let cropped = wm_try!(image.crop_imm(geom.xoffset, geom.yoffset, geom.width, geom.height));
    *image = cropped;
    Ok(())
}
