use crate::{arg_parse_err::ArgParseErr, arg_parsers::strip_and_parse_number};
use std::{ffi::OsStr, str::FromStr};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct UnsharpGeometry {
    radius: usize, // unused, only to satisfy ImageMagick's CLI
    pub sigma: f32,
    gain: f32,          // unused, only to satisfy ImageMagick's CLI
    pub threshold: i32, // doesn't match the ImageMagick type of floating point
}

impl FromStr for UnsharpGeometry {
    type Err = ArgParseErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(OsStr::new(s))
    }
}

impl TryFrom<&OsStr> for UnsharpGeometry {
    type Error = ArgParseErr;

    fn try_from(s: &OsStr) -> Result<Self, Self::Error> {
        if s.is_empty() {
            return Err(ArgParseErr::with_msg(
                "blur geometry must be non-empty and ASCII only",
            ));
        }

        Ok(Self {
            radius: 0,
            sigma: 1.0,
            gain: 1.0,
            threshold: 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::UnsharpGeometry;
    use std::str::FromStr;

    #[test]
    fn test_radius_only() {
        let geom = UnsharpGeometry::from_str("5").unwrap();
        assert_eq!(
            geom,
            UnsharpGeometry {
                radius: 5,
                ..Default::default()
            }
        );
    }

    #[test]
    fn test_radius_and_sigma() {
        let geom = UnsharpGeometry::from_str("5x1.1").unwrap();
        assert_eq!(
            geom,
            UnsharpGeometry {
                radius: 5,
                sigma: 1.1,
                ..Default::default()
            }
        );
    }

    #[test]
    fn test_radius_and_sigma_and_gain() {
        let geom = UnsharpGeometry::from_str("2x1.5+9").unwrap();
        assert_eq!(
            geom,
            UnsharpGeometry {
                radius: 2,
                sigma: 1.5,
                gain: 9.0,
                ..Default::default()
            }
        );
    }
    #[test]
    fn test_radius_and_sigma_and_gain_and_threshold() {
        let geom = UnsharpGeometry::from_str("42x2.1+7+11").unwrap();
        assert_eq!(
            geom,
            UnsharpGeometry {
                radius: 42,
                sigma: 2.1,
                gain: 7.0,
                threshold: 11,
            }
        );
    }

    #[test]
    fn test_invalid() {
        assert!(UnsharpGeometry::from_str("💥 not ascii only").is_err());
        assert!(UnsharpGeometry::from_str("").is_err());
        assert!(UnsharpGeometry::from_str("abc").is_err());
        assert!(UnsharpGeometry::from_str("0x0x0x0x").is_err());
    }
}
