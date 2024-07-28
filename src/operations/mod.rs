mod crop;
mod resize;

use image::DynamicImage;

use crate::{
    arg_parsers::{LoadCropGeometry, ResizeGeometry},
    error::MagickError,
};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Operation {
    Resize(ResizeGeometry),
    Thumbnail(ResizeGeometry),
    Scale(ResizeGeometry),
    Sample(ResizeGeometry),
    CropOnLoad(LoadCropGeometry),
}

impl Operation {
    pub fn execute(&self, image: &mut DynamicImage) -> Result<(), MagickError> {
        match self {
            Operation::Resize(geom) => resize::resize(image, geom),
            Operation::Thumbnail(geom) => resize::thumbnail(image, geom),
            Operation::Scale(geom) => resize::scale(image, geom),
            Operation::Sample(geom) => resize::sample(image, geom),
            Operation::CropOnLoad(geom) => crop::crop_on_load(image, geom),
        }
    }
}
