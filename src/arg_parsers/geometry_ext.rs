// Fun fact: the geometry documentation at https://www.imagemagick.org/Magick++/Geometry.html is a lie.
//
// It says things like
// > Offsets must be given as pairs; in other words, in order to specify either xoffset or yoffset both must be present.
// but this works:
// `convert rose: -crop 50x+0 crop_half.gif`
//
// It also says:
// > Extended geometry strings should *only* be used when *resizing an image.*
// but this works:
// `convert rose: -crop 50% crop_half.gif`
//
// So we just rely on observing the actual behavior of `convert` instead.

use crate::arg_parsers::Geometry;

/// Intermediate result of extended geometry parsing
///
/// Imagemagick uses the same parser for all [extended geometry](https://www.imagemagick.org/Magick++/Geometry.html).
/// Parsing is implemented on this struct, and we convert it into more specific structs like [ResizeGeometry] later.
#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct ExtendedGeometry {
    flags: ExtendedGeometryFlags,
    geom: Geometry,
}

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct ExtendedGeometryFlags {
    ignore_aspect_ratio: bool,
    percentage_mode: bool,
    area_mode: bool,
    cover_mode: bool,
    only_enlarge: bool,
    only_shrink: bool,
}
