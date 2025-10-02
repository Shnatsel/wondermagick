//! Parsers for specific command-line argument formats,
//! e.g. <https://www.imagemagick.org/Magick++/Geometry.html>

mod crop;
pub use crop::*;
mod resize;
pub use resize::*;
mod geometry;
pub(self) use geometry::*;
mod geometry_ext;
pub(self) use geometry_ext::*;
mod filename;
pub use filename::*;
mod numbers;
pub use numbers::*;
mod identify_format;
pub use identify_format::*;
