use crate::arg_parsers::{CropGeometry, LoadCropGeometry};
use crate::image::Image;

pub fn crop_on_load(
    image: &mut Image,
    geom: &LoadCropGeometry,
) -> Result<(), crate::error::MagickError> {
    let image = &mut image.pixels;
    // Sadly this doesn't check bounds right now, so we can get panics later on because of wrong crop parameters:
    // https://github.com/image-rs/image/issues/2296
    // TODO: change this in `image` because I don't want to emulate this on the client side
    // and pretend the problem doesn't exist for anyone else
    let rect = image::math::Rect {
        x: geom.xoffset,
        y: geom.yoffset,
        width: geom.width,
        height: geom.height,
    };
    // crop_in_place exists but doesn't shrink the backing buffer,
    // so we have to either allocate the large buffer and keep it around forever
    // (lower peak memory usage) or allocate both large and small buffer but drop the large one
    // (higher peak memory usage, quickly goes down later).
    // This takes the second option, but we might reconsider later if there's compelling evidence.
    let cropped = image.crop(rect);
    *image = cropped;
    Ok(())
}

pub fn crop(image: &mut Image, geom: &CropGeometry) -> Result<(), crate::error::MagickError> {
    // TODO: lots of flags and edge cases
    if geom.percentage_mode {
        panic!("Percentage crop is not yet implemented")
    }

    let area = geom.area;

    let new_geom = LoadCropGeometry {
        width: area.width.unwrap_or(image.pixels.width()),
        height: area.height.unwrap_or(image.pixels.height()),
        xoffset: area
            .xoffset
            .unwrap_or(0)
            .try_into()
            .expect("negative crop offsets are not yet implemented"),
        yoffset: area
            .yoffset
            .unwrap_or(0)
            .try_into()
            .expect("negative crop offsets are not yet implemented"),
    };

    crop_on_load(image, &new_geom)
}
