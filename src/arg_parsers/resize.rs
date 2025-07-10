use std::{
    ffi::OsStr,
    fmt::{Display, Write},
    str::FromStr,
};

#[cfg(test)]
use crate::utils::arbitrary;
#[cfg(test)]
use derive_quickcheck_arbitrary::Arbitrary;

use crate::{arg_parse_err::ArgParseErr, arg_parsers::ExtGeometry};

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
        #[cfg_attr(test, arbitrary(gen(|g| arbitrary::optional_positive_float(g) )))]
        width: Option<f64>,
        #[cfg_attr(test, arbitrary(gen(|g| arbitrary::positive_float(g) )))]
        height: f64,
    },
    /// `@` operator
    Area(
        // Do not generate unrealistically huge image areas above 16TiB that run into float precision issues
        #[cfg_attr(test, arbitrary(gen(|g| u64::arbitrary(g).clamp(0, u64::MAX / 1024 / 1024) )))]
        u64,
    ),
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
                write!(f, "x")?;
                if let Some(h) = height {
                    write!(f, "{}", h)?;
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
    type Err = ArgParseErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(OsStr::new(s))
    }
}

impl TryFrom<&OsStr> for ResizeGeometry {
    type Error = ArgParseErr;

    fn try_from(s: &OsStr) -> Result<Self, Self::Error> {
        if !s.is_ascii() {
            return Err(ArgParseErr::new());
        }

        let geom_ext = ExtGeometry::try_from(s)?;
        let geom = geom_ext.geom;
        let flags = geom_ext.flags;

        let ignore_aspect_ratio = flags.exclamation;
        let percentage_mode = flags.percent;
        let area_mode = flags.at;
        let cover_mode = flags.caret;
        let only_enlarge = flags.less_than;
        let only_shrink = flags.greater_than;

        if only_enlarge && only_shrink {
            return Err(ArgParseErr::with_msg(
                "< and > cannot be specified together",
            ));
        }
        let mut constraint = ResizeConstraint::default();
        if only_enlarge {
            constraint = ResizeConstraint::OnlyEnlarge;
        } else if only_shrink {
            constraint = ResizeConstraint::OnlyShrink;
        }

        let mut target = ResizeTarget::default();
        // If both percentage and area are specified, area takes precedence.
        if area_mode {
            if let Some(width) = geom.width {
                // height is ignored, I've checked
                target = ResizeTarget::Area(width.round() as u64);
            } else {
                // imagemagick reports "negative or zero image size" followed by the path to the image, and frankly fuck that
                return Err(ArgParseErr::with_msg(
                    "please specify the area to resize to when using @ operator",
                ));
            }
        } else if percentage_mode {
            match (geom.width, geom.height) {
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
            if geom.width.is_some() || geom.height.is_some() {
                // convert to integers
                let width = geom.width.map(|f| f.round() as u32);
                let height = geom.height.map(|f| f.round() as u32);
                // passing any single dimension (width or height) will cause imagemagick
                // to apply this rule to both dimensions
                let width = width.unwrap_or_else(|| height.unwrap());
                let height = height.unwrap_or(width);
                target = ResizeTarget::FullyCover { width, height }
            }
        } else {
            // convert floats that imagemagick FOR SOME REASON accepts as dimensions to integers
            let width = geom.width.map(|f| f.round() as u32); // imagemagick rounds to nearest
            let height = geom.height.map(|f| f.round() as u32); // imagemagick rounds to nearest
            target = ResizeTarget::Size {
                width,
                height,
                ignore_aspect_ratio,
            };
        }

        Ok(Self { target, constraint })
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
        let stringified = orig.to_string();
        let parsed = ResizeGeometry::from_str(&stringified).unwrap();
        assert_eq!(orig, parsed)
    }
}
