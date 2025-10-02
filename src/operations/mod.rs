mod auto_orient;
mod crop;
mod identify;
mod resize;

use crate::{
    arg_parsers::{CropGeometry, IdentifyFormat, LoadCropGeometry, ResizeGeometry},
    error::MagickError,
    image::Image,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Operation {
    Resize(ResizeGeometry),
    Thumbnail(ResizeGeometry),
    Scale(ResizeGeometry),
    Sample(ResizeGeometry),
    CropOnLoad(LoadCropGeometry),
    Crop(CropGeometry),
    Identify(IdentifyFormat),
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
            Operation::Crop(geom) => crop::crop(image, geom),
            Operation::Identify(format) => identify::identify(image, format.clone()),
            Operation::AutoOrient => auto_orient::auto_orient(image),
        }
    }
}
