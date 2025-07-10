mod auto_orient;
mod crop;
mod resize;

use crate::{
    arg_parsers::{CropGeometry, LoadCropGeometry, ResizeGeometry},
    error::MagickError,
    image::Image,
};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Operation {
    Resize(ResizeGeometry),
    Thumbnail(ResizeGeometry),
    Scale(ResizeGeometry),
    Sample(ResizeGeometry),
    CropOnLoad(LoadCropGeometry),
    Crop(CropGeometry),
    AutoOrient,
}

impl Operation {
    pub fn execute(&self, image: &mut Image) -> Result<(), MagickError> {
        match self {
            Operation::Resize(geom) => resize::resize(image, geom),
            Operation::Thumbnail(geom) => resize::thumbnail(image, geom),
            Operation::Scale(geom) => resize::scale(image, geom),
            Operation::Sample(geom) => resize::sample(image, geom),
            Operation::CropOnLoad(geom) => crop::crop_on_load(image, geom),
            Operation::Crop(geom) => todo!(),
            Operation::AutoOrient => auto_orient::auto_orient(image),
        }
    }
}
