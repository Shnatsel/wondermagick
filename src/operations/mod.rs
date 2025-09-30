mod auto_orient;
mod crop;
mod identify;
mod resize;

use crate::{
    arg_parsers::{CropGeometry, LoadCropGeometry, ResizeGeometry},
    error::MagickError,
    image::Image,
};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Operation {
    AutoOrient,
    Crop(CropGeometry),
    CropOnLoad(LoadCropGeometry),
    Identify,
    Resize(ResizeGeometry),
    Sample(ResizeGeometry),
    Scale(ResizeGeometry),
    Thumbnail(ResizeGeometry),
}

impl Operation {
    pub fn execute(&self, image: &mut Image) -> Result<(), MagickError> {
        match self {
            Operation::AutoOrient => auto_orient::auto_orient(image),
            Operation::Crop(geom) => crop::crop(image, geom),
            Operation::CropOnLoad(geom) => crop::crop_on_load(image, geom),
            Operation::Identify => {
                let info = identify::identify(image)
                    .unwrap_or_else(|e| format!("Failed to identify image: {}", e));
                println!("{}", info);
                Ok(())
            }
            Operation::Resize(geom) => resize::resize(image, geom),
            Operation::Sample(geom) => resize::sample(image, geom),
            Operation::Scale(geom) => resize::scale(image, geom),
            Operation::Thumbnail(geom) => resize::thumbnail(image, geom),
        }
    }
}
