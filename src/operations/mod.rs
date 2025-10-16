mod auto_orient;
mod composite;
mod crop;
mod identify;
mod resize;

use crate::{
    arg_parsers::{CropGeometry, IdentifyFormat, LoadCropGeometry, ResizeGeometry},
    decode::decode,
    error::MagickError,
    image::Image,
    plan::FilePlan,
    wm_try,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Operation {
    Resize(ResizeGeometry),
    Thumbnail(ResizeGeometry),
    Scale(ResizeGeometry),
    Sample(ResizeGeometry),
    Composite(FilePlan, Option<FilePlan>),
    CropOnLoad(LoadCropGeometry),
    Crop(CropGeometry),
    Identify(Option<IdentifyFormat>),
    AutoOrient,
}

impl Operation {
    pub fn execute(&self, image: &mut Image) -> Result<(), MagickError> {
        match self {
            Operation::Resize(geom) => resize::resize(image, geom),
            Operation::Thumbnail(geom) => resize::thumbnail(image, geom),
            Operation::Scale(geom) => resize::scale(image, geom),
            Operation::Sample(geom) => resize::sample(image, geom),
            Operation::Composite(other_image, _mask) => {
                let other_image = wm_try!(decode(&other_image.location, other_image.format));
                composite::composite(image, &other_image, None)
            }
            Operation::CropOnLoad(geom) => crop::crop_on_load(image, geom),
            Operation::Crop(geom) => crop::crop(image, geom),
            Operation::Identify(format) => identify::identify(image, format.clone()),
            Operation::AutoOrient => auto_orient::auto_orient(image),
        }
    }
}
