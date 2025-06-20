//! Parsers for specific command-line argument formats,
//! e.g. <https://www.imagemagick.org/Magick++/Geometry.html>

mod resize;
pub use resize::*;
mod geometry;
pub use geometry::*;
mod filename;
pub use filename::*;
mod quality;
pub mod numbers;