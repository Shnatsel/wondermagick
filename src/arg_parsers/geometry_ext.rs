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

use std::{ffi::OsStr, str::FromStr};

use crate::{arg_parse_err::ArgParseErr, arg_parsers::Geometry};

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct ExtGeometryFlags {
    /// !
    pub exclamation: bool,
    /// %
    pub percent: bool,
    /// @
    pub at: bool,
    /// ^
    pub caret: bool,
    /// <
    pub less_than: bool,
    /// >
    pub greater_than: bool,
}

/// Intermediate result of extended geometry parsing
///
/// Imagemagick uses the same parser for all [extended geometry](https://www.imagemagick.org/Magick++/Geometry.html).
/// Parsing is implemented on this struct, and we convert it into more specific structs like [ResizeGeometry] later.
#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct ExtGeometry {
    pub geom: Geometry,
    pub flags: ExtGeometryFlags,
}

impl TryFrom<&OsStr> for ExtGeometry {
    type Error = ArgParseErr;

    fn try_from(s: &OsStr) -> Result<Self, Self::Error> {
        if !s.is_ascii() {
            return Err(ArgParseErr::new());
        }

        let ascii = s.as_encoded_bytes();
        let mut ascii = ascii.to_vec();
        let orig_len = ascii.len();

        // "50!0!x+0!+0" parses as "500x+0+0",
        // so my guess is that imagemagick scans for special characters and removes them,
        // then parses the rest as a regular geometry.

        let flags = ExtGeometryFlags {
            exclamation: find_and_remove_byte(b'!', &mut ascii),
            percent: find_and_remove_byte(b'%', &mut ascii),
            at: find_and_remove_byte(b'@', &mut ascii),
            caret: find_and_remove_byte(b'^', &mut ascii),
            less_than: find_and_remove_byte(b'<', &mut ascii),
            greater_than: find_and_remove_byte(b'>', &mut ascii),
        };

        let geom = if orig_len > 0 && ascii.len() == 0 {
            Geometry::default()
            // imagemagick permits extended geometry with only one symbol such as @ or ^ and nothing else, which is a no-op
        } else {
            // attempt to parse the remainder as a geometry, which will fail for an emtry string
            let geom_str = std::str::from_utf8(&ascii).unwrap(); // it's ascii, should never panic
            Geometry::from_str(geom_str)?
        };

        Ok(Self { flags, geom })
    }
}

/// Returns whether the specified byte was found. Keeps the slice intact if not.
fn find_and_remove_byte(byte: u8, vec: &mut Vec<u8>) -> bool {
    let orig_len = vec.len();
    vec.retain(|elem| *elem != byte);
    vec.len() != orig_len
}

// no tests in this module because this parser underpins resize geometry parsing and we test it through that
