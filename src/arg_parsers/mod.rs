//! Parsers for specific command-line argument formats,
//! e.g. <https://www.imagemagick.org/Magick++/Geometry.html>

mod crop;
pub use crop::*;
mod resize;
pub use resize::*;
mod geometry;
use geometry::*;
mod geometry_ext;
use geometry_ext::*;
mod filename;
pub use filename::*;
mod numbers;
pub use numbers::*;
mod identify_format;
pub use identify_format::{IdentifyFormat, Token, Var};
mod filter;
pub use filter::*;
mod blur_geometry;
pub use blur_geometry::*;
mod grayscale_method;
pub use grayscale_method::*;
mod unsharp_geometry;
pub use unsharp_geometry::*;
