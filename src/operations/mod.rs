mod auto_orient;
mod blur;
mod combine;
mod crop;
mod flip;
pub use flip::Axis;
mod grayscale;
mod identify;
mod monochrome;
mod negate;
mod resize;
mod unsharpen;

use crate::{
    arg_parsers::{
        BlurGeometry, CropGeometry, Filter, GrayscaleMethod, IdentifyFormat, LoadCropGeometry,
        ResizeGeometry, UnsharpenGeometry,
    },
    error::MagickError,
    image::Image,
    plan,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Operation {
    Resize(ResizeGeometry, Option<Filter>),
    Thumbnail(ResizeGeometry, Option<Filter>),
    Scale(ResizeGeometry),
    Sample(ResizeGeometry),
    CropOnLoad(LoadCropGeometry),
    Crop(CropGeometry),
    Identify(Option<IdentifyFormat>),
    Negate,
    AutoOrient,
    Blur(BlurGeometry),
    GaussianBlur(BlurGeometry),
    Grayscale(GrayscaleMethod),
    Flip(Axis),
    Monochrome,
    Unsharpen(UnsharpenGeometry),
}

impl Operation {
    pub fn execute(&self, image: &mut Image) -> Result<(), MagickError> {
        match self {
            Operation::Resize(geom, filter) => resize::resize(image, geom, *filter),
            Operation::Thumbnail(geom, filter) => resize::thumbnail(image, geom, *filter),
            Operation::Scale(geom) => resize::scale(image, geom),
            Operation::Sample(geom) => resize::sample(image, geom),
            Operation::CropOnLoad(geom) => crop::crop_on_load(image, geom),
            Operation::Crop(geom) => crop::crop(image, geom),
            Operation::Identify(format) => identify::identify(image, format.clone()),
            Operation::Negate => negate::negate(image),
            Operation::AutoOrient => auto_orient::auto_orient(image),
            Operation::Blur(geom) => blur::blur(image, geom),
            Operation::GaussianBlur(geom) => blur::gaussian_blur(image, geom),
            Operation::Grayscale(method) => grayscale::grayscale(image, method),
            Operation::Flip(axis) => flip::flip(image, axis),
            Operation::Monochrome => monochrome::monochrome(image),
            Operation::Unsharpen(geom) => unsharpen::unsharpen(image, geom),
        }
    }

    /// Modifiers are flags such as -quality that affect operations.
    /// For global operations we need to alter them after the operation's creation,
    /// to apply up-to-date modifiers.
    pub fn apply_modifiers(&mut self, mods: &plan::Modifiers) {
        use Operation::*;
        match self {
            Resize(resize_geometry, _) => *self = Resize(*resize_geometry, mods.filter),
            Thumbnail(resize_geometry, _) => *self = Thumbnail(*resize_geometry, mods.filter),
            Scale(_) => (),
            Sample(_) => (),
            CropOnLoad(_) => (),
            Crop(_) => (),
            Identify(_) => *self = Identify(mods.identify_format.clone()),
            Negate => (),
            AutoOrient => (),
            Blur(_) => (),
            GaussianBlur(_) => (),
            Grayscale(_) => (),
            Flip(_) => (),
            Monochrome => (),
            Unsharpen(_) => (),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RewriteOperation {
    Combine {
        color: image::ColorType,
        /// Rewrite the color model to true color (`sRGB`) when the channel count is exceeded?
        fallback_for_channel_count: bool,
    },
}

impl RewriteOperation {
    pub(crate) fn execute(&self, sequence: &mut Vec<Image>) -> Result<(), MagickError> {
        match self {
            &RewriteOperation::Combine {
                color,
                fallback_for_channel_count,
            } => {
                let image =
                    combine::combine(sequence.split_off(0), color, fallback_for_channel_count)?;
                sequence.push(image);
                Ok(())
            }
        }
    }
}
