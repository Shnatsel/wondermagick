use std::{
    ffi::OsStr,
    fmt::{Display, Write},
    num::ParseFloatError,
    str::FromStr,
};

#[cfg(test)]
use derive_quickcheck_arbitrary::Arbitrary;

use crate::{error::MagickError, wm_err, wm_try};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum ResizeConstraint {
    #[default]
    Unconstrained,
    OnlyEnlarge,
    OnlyShrink,
}

impl From<&ResizeConstraint> for &'static str {
    fn from(value: &ResizeConstraint) -> Self {
        match value {
            ResizeConstraint::Unconstrained => "",
            ResizeConstraint::OnlyEnlarge => "<",
            ResizeConstraint::OnlyShrink => ">",
        }
    }
}

impl Display for ResizeConstraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string: &'static str = self.into();
        f.write_str(string)
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum ResizeTarget {
    Size {
        width: Option<u32>,
        height: Option<u32>,
        /// `!` operator
        ignore_aspect_ratio: bool,
    },
    /// `%` operator
    Percentage {
        #[cfg_attr(test, arbitrary(gen(|g| if bool::arbitrary(g) {Some(arbitrary_nonnegative_float(g))} else {None} )))]
        width: Option<f64>,
        #[cfg_attr(test, arbitrary(gen(|g| arbitrary_nonnegative_float(g) )))]
        height: f64,
    },
    /// `@` operator
    Area(u64),
    /// ^` operator
    FullyCover { width: u32, height: u32 },
}

impl Display for ResizeTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResizeTarget::Size {
                width,
                height,
                ignore_aspect_ratio,
            } => {
                if let Some(w) = width {
                    write!(f, "{}", w)?;
                }
                if let Some(h) = height {
                    write!(f, "x{}", h)?;
                }
                if *ignore_aspect_ratio {
                    f.write_char('!')?;
                }
                Ok(())
            }
            ResizeTarget::Percentage { width, height } => {
                if let Some(w) = width {
                    write!(f, "{}", w)?;
                }
                write!(f, "x{}%", height)
            }
            ResizeTarget::Area(area) => {
                write!(f, "@{}", area)
            }
            ResizeTarget::FullyCover { width, height } => {
                write!(f, "^{}x{}", width, height)
            }
        }
    }
}

impl Default for ResizeTarget {
    fn default() -> Self {
        Self::Size {
            width: None,
            height: None,
            ignore_aspect_ratio: false,
        }
    }
}

/// "Extended geometry" according to imagemagick docs. Only used in resizing operations.
#[derive(Copy, Clone, PartialEq, Debug, Default)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct ResizeGeometry {
    pub target: ResizeTarget,
    pub constraint: ResizeConstraint,
}

impl Display for ResizeGeometry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.target, self.constraint)
    }
}

impl FromStr for ResizeGeometry {
    type Err = MagickError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(OsStr::new(s))
    }
}

impl TryFrom<&OsStr> for ResizeGeometry {
    type Error = MagickError;

    fn try_from(s: &OsStr) -> Result<Self, Self::Error> {
        if !s.is_ascii() {
            return Err(wm_err!(
                "invalid argument for option `-resize': {}",
                s.to_string_lossy()
            ));
        }
        let ascii = s.as_encoded_bytes();

        let ignore_aspect_ratio = ascii.contains(&b'!');
        let percentage_mode = ascii.contains(&b'%');
        let area_mode = ascii.contains(&b'@');
        let cover_mode = ascii.contains(&b'^');
        let only_enlarge = ascii.contains(&b'<');
        let only_shrink = ascii.contains(&b'>');
        if only_enlarge && only_shrink {
            return Err(wm_err!(
                "invalid argument for option `-resize': < and > cannot be specified together"
            ));
        }
        let mut constraint = ResizeConstraint::default();
        if only_enlarge {
            constraint = ResizeConstraint::OnlyEnlarge;
        } else if only_shrink {
            constraint = ResizeConstraint::OnlyShrink;
        }

        let mut iter = ascii.split(|c| *c == b'x');
        let width = if let Some(slice) = iter.next() {
            wm_try!(find_and_parse_float(slice))
        } else {
            None
        };
        let height = if let Some(slice) = iter.next() {
            wm_try!(find_and_parse_float(slice))
        } else {
            None
        };

        let mut target = ResizeTarget::default();
        // If both percentage and area are specified, area takes precedence.
        if area_mode {
            if let Some(width) = width {
                // height is ignored, I've checked
                target = ResizeTarget::Area(width.round() as u64);
            } else {
                // imagemagick reports "negative or zero image size" followed by the path to the image, and frankly fuck that
                return Err(wm_err!(
                    "please specify the area to resize to when using @ operator"
                ));
            }
        } else if percentage_mode {
            match (width, height) {
                (None, None) => {} // imagemagick accepts % without a number, which amounts to a no-op
                (Some(width), None) => {
                    // width but not height being specified means the same scaling applies to both axes
                    target = ResizeTarget::Percentage {
                        width: Some(width),
                        height: width,
                    }
                }
                (None, Some(height)) => {
                    // Only height being specified means we only scale height AND ignore aspect ratio.
                    // I could not find an equivalent mode to scale width only.
                    target = ResizeTarget::Percentage {
                        width: None,
                        height,
                    }
                }
                (Some(width), Some(height)) => {
                    // imagemagick ignores aspect ratio in this case
                    target = ResizeTarget::Percentage {
                        width: Some(width),
                        height,
                    }
                }
            }
        } else if cover_mode {
            // simply passing ^ without any digits is treated as a no-op and not rejected
            if width.is_some() || height.is_some() {
                // convert to integers
                let width = width.map(|f| f.round() as u32);
                let height = height.map(|f| f.round() as u32);
                // passing any single dimension (width or height) will cause imagemagick
                // to apply this rule to both dimensions
                let width = width.unwrap_or_else(|| height.unwrap());
                let height = height.unwrap_or(width);
                target = ResizeTarget::FullyCover { width, height }
            }
        } else {
            // don't even set any flags if this is a no-op
            if width.is_some() || height.is_some() {
                // convert floats that imagemagick FOR SOME REASON accepts as dimensions to integers
                let width = width.map(|f| f.round() as u32); // imagemagick rounds to nearest
                let height = height.map(|f| f.round() as u32); // imagemagick rounds to nearest
                target = ResizeTarget::Size {
                    width,
                    height,
                    ignore_aspect_ratio,
                };
            }
        }

        // The offsets after the resolution, such as +500 or -200, are accepted by the imagemagick parser but ignored.
        // We don't even bother parsing them.

        Ok(ResizeGeometry { target, constraint })
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

#[must_use]
#[cfg(test)]
fn arbitrary_nonnegative_float(gen: &mut quickcheck::Gen) -> f64 {
    use quickcheck::Arbitrary;
    let raw = f64::arbitrary(gen).abs();
    if raw.is_infinite() || raw.is_nan() {
        0.0
    } else {
        raw
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use quickcheck_macros::quickcheck;

    use crate::arg_parsers::{resize::ResizeConstraint, ResizeTarget};

    use super::ResizeGeometry;

    #[test]
    fn test_width_only() {
        let mut expected = ResizeGeometry::default();
        expected.target = ResizeTarget::Size {
            width: Some(40),
            height: None,
            ignore_aspect_ratio: false,
        };
        let parsed = ResizeGeometry::from_str("40").unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_height_only() {
        let mut expected = ResizeGeometry::default();
        expected.target = ResizeTarget::Size {
            width: None,
            height: Some(50),
            ignore_aspect_ratio: false,
        };
        let parsed = ResizeGeometry::from_str("x50").unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_ignore_aspect_ratio() {
        let mut expected = ResizeGeometry::default();
        expected.target = ResizeTarget::Size {
            width: Some(40),
            height: Some(50),
            ignore_aspect_ratio: true,
        };
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
        expected.target = ResizeTarget::Size {
            width: Some(40),
            height: Some(50),
            ignore_aspect_ratio: false,
        };
        expected.constraint = ResizeConstraint::OnlyEnlarge;
        let parsed = ResizeGeometry::from_str("<40x50").unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_only_enlarge_width() {
        let mut expected = ResizeGeometry::default();
        expected.target = ResizeTarget::Size {
            width: Some(40),
            height: None,
            ignore_aspect_ratio: false,
        };
        expected.constraint = ResizeConstraint::OnlyEnlarge;
        let parsed = ResizeGeometry::from_str("<40").unwrap();
        assert_eq!(parsed, expected);
        let parsed = ResizeGeometry::from_str("40<").unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_only_enlarge_height() {
        let mut expected = ResizeGeometry::default();
        expected.target = ResizeTarget::Size {
            width: None,
            height: Some(50),
            ignore_aspect_ratio: false,
        };
        expected.constraint = ResizeConstraint::OnlyEnlarge;
        let parsed = ResizeGeometry::from_str("<x50").unwrap();
        assert_eq!(parsed, expected);
        let parsed = ResizeGeometry::from_str("x50<").unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_only_shrink() {
        let mut expected = ResizeGeometry::default();
        expected.target = ResizeTarget::Size {
            width: Some(40),
            height: Some(50),
            ignore_aspect_ratio: false,
        };
        expected.constraint = ResizeConstraint::OnlyShrink;
        let parsed = ResizeGeometry::from_str(">40x50").unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_ignored_offsets() {
        let mut expected = ResizeGeometry::default();
        expected.target = ResizeTarget::Size {
            width: Some(40),
            height: Some(50),
            ignore_aspect_ratio: false,
        };
        let parsed = ResizeGeometry::from_str("40x50-60").unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_percentage() {
        let mut expected = ResizeGeometry::default();
        expected.target = ResizeTarget::Percentage {
            width: Some(40.0),
            height: 40.0,
        };
        let parsed = ResizeGeometry::from_str("40%").unwrap();
        assert_eq!(parsed, expected);
        let parsed = ResizeGeometry::from_str("%40").unwrap();
        assert_eq!(parsed, expected);
        let parsed = ResizeGeometry::from_str("40x40%").unwrap();
        assert_eq!(parsed, expected);
        let parsed = ResizeGeometry::from_str("%40%x%40%").unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_percentage_different_width_height() {
        let mut expected = ResizeGeometry::default();
        expected.target = ResizeTarget::Percentage {
            width: Some(40.0),
            height: 50.0,
        };
        let parsed = ResizeGeometry::from_str("40x50%").unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_percentage_only_height() {
        let mut expected = ResizeGeometry::default();
        expected.target = ResizeTarget::Percentage {
            width: None,
            height: 50.0,
        };
        let parsed = ResizeGeometry::from_str("x50%").unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_percentage_no_op() {
        let expected = ResizeGeometry::default();
        let parsed = ResizeGeometry::from_str("%").unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_area() {
        let mut expected = ResizeGeometry::default();
        expected.target = ResizeTarget::Area(200);
        let parsed = ResizeGeometry::from_str("200@").unwrap();
        assert_eq!(parsed, expected);
        let parsed = ResizeGeometry::from_str("@200").unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_area_with_ignored_height() {
        let mut expected = ResizeGeometry::default();
        expected.target = ResizeTarget::Area(200);
        let parsed = ResizeGeometry::from_str("200x500@").unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_cover_no_op() {
        let expected = ResizeGeometry::default();
        let parsed = ResizeGeometry::from_str("^").unwrap();
        assert_eq!(parsed, expected);
    }

    #[quickcheck]
    fn roundtrip_is_lossless(orig: ResizeGeometry) {
        // Skip unrealistically huge image areas that run into float precision issues
        if let ResizeTarget::Area(area) = orig.target {
            let huge_16tb_image_area = u64::MAX / 1024 / 1024;
            if area >= huge_16tb_image_area {
                return;
            }
        }

        let stringified = orig.to_string();
        let parsed = ResizeGeometry::from_str(&stringified).unwrap();
        assert_eq!(orig, parsed)
    }
}
