use std::{ffi::OsStr, num::ParseFloatError, str::FromStr};

use crate::args::ArgParseErr;

/// Extended geometry
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
pub struct ResizeGeometry {
    pub width: Option<u64>,
    pub height: Option<u64>,
    pub ignore_aspect_ratio: bool,
    pub only_enlarge: bool,
    pub only_shrink: bool,
    // TODO: percentage mode, which can be fractional
    // TODO: area mode
}

impl FromStr for ResizeGeometry {
    type Err = ArgParseErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(OsStr::new(s))
    }
}

impl TryFrom<&OsStr> for ResizeGeometry {
    type Error = ArgParseErr;

    fn try_from(s: &OsStr) -> Result<Self, Self::Error> {
        // TODO: support all of these qualifiers: https://www.imagemagick.org/Magick++/Geometry.html
        // TODO: return default value (no-op) on certain mailformed strings like imagemagick does
        if !s.is_ascii() {
            return Err(ArgParseErr {});
        }
        let ascii = s.as_encoded_bytes();

        let ignore_aspect_ratio = ascii.contains(&b'!');
        let only_enlarge = ascii.contains(&b'<');
        let only_shrink = ascii.contains(&b'>');
        if only_enlarge && only_shrink {
            return Err(ArgParseErr {});
        }

        let mut iter = ascii.split(|c| *c == b'x');
        let width = if let Some(slice) = iter.next() {
            find_and_parse_float(slice)
                .map_err(|_| ArgParseErr {})?
                .map(|f| f.round() as u64) // imagemagick rounds to nearest
        } else {
            None
        };
        let height = if let Some(slice) = iter.next() {
            find_and_parse_float(slice)
                .map_err(|_| ArgParseErr {})?
                .map(|f| f.round() as u64) // imagemagick rounds to nearest
        } else {
            None
        };

        // The offsets after the resolution, such as +500 or -200, are accepted by the imagemagick parser but ignored.
        // We don't even bother parsing them.

        Ok(ResizeGeometry {
            width,
            height,
            ignore_aspect_ratio,
            only_enlarge,
            only_shrink,
        })
    }
}

fn find_and_parse_float(input: &[u8]) -> Result<Option<f64>, ParseFloatError> {
    // Yes, imagemagick accepts floating-point image dimensions for resizing.
    // No, I don't know why either.
    let number: Vec<u8> = input
        .iter()
        .copied()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit() || *c == b'.')
        .collect();
    if number.is_empty() {
        Ok(None)
    } else {
        let number_str = String::from_utf8(number).unwrap();
        number_str.parse::<f64>().map(|f| Some(f))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::ResizeGeometry;

    #[test]
    fn test_width_only() {
        let mut expected = ResizeGeometry::default();
        expected.width = Some(40);
        let parsed = ResizeGeometry::from_str("40").unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_height_only() {
        let mut expected = ResizeGeometry::default();
        expected.height = Some(50);
        let parsed = ResizeGeometry::from_str("x50").unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_ignore_aspect_ratio() {
        let mut expected = ResizeGeometry::default();
        expected.width = Some(40);
        expected.height = Some(50);
        expected.ignore_aspect_ratio = true;
        let parsed = ResizeGeometry::from_str("!40x50").unwrap();
        assert_eq!(parsed, expected);
        let parsed = ResizeGeometry::from_str("40x50!").unwrap();
        assert_eq!(parsed, expected);
        let parsed = ResizeGeometry::from_str("40!x50").unwrap();
        assert_eq!(parsed, expected);
        let parsed = ResizeGeometry::from_str("40x!50").unwrap();
        assert_eq!(parsed, expected);
        let parsed = ResizeGeometry::from_str("!40!x!50!").unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_only_enlarge() {
        let mut expected = ResizeGeometry::default();
        expected.width = Some(40);
        expected.height = Some(50);
        expected.only_enlarge = true;
        let parsed = ResizeGeometry::from_str("<40x50").unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_only_enlarge_width() {
        let mut expected = ResizeGeometry::default();
        expected.width = Some(40);
        expected.only_enlarge = true;
        let parsed = ResizeGeometry::from_str("<40").unwrap();
        assert_eq!(parsed, expected);
        let parsed = ResizeGeometry::from_str("40<").unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_only_enlarge_height() {
        let mut expected = ResizeGeometry::default();
        expected.height = Some(50);
        expected.only_enlarge = true;
        let parsed = ResizeGeometry::from_str("<x50").unwrap();
        assert_eq!(parsed, expected);
        let parsed = ResizeGeometry::from_str("x50<").unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_ignored_offsets() {
        let mut expected = ResizeGeometry::default();
        expected.width = Some(40);
        expected.height = Some(50);
        let parsed = ResizeGeometry::from_str("40x50-60").unwrap();
        assert_eq!(parsed, expected);
    }
}
