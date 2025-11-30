use crate::{arg_parse_err::ArgParseErr, arg_parsers::strip_and_parse_number};
use std::{ffi::OsStr, str::FromStr};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct UnsharpenGeometry {
    radius: usize, // unused, only present to satisfy ImageMagick's CLI
    pub sigma: f32,
    gain: f32,          // unused, only present to satisfy ImageMagick's CLI
    pub threshold: i32, // doesn't match the ImageMagick type of floating point
}

impl FromStr for UnsharpenGeometry {
    type Err = ArgParseErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(OsStr::new(s))
    }
}

impl TryFrom<&OsStr> for UnsharpenGeometry {
    type Error = ArgParseErr;

    fn try_from(s: &OsStr) -> Result<Self, Self::Error> {
        if s.is_empty() {
            return Err(ArgParseErr::with_msg(
                "unsharp geometry must be non-empty and ASCII only",
            ));
        }

        let string: &str = s
            .try_into()
            .map_err(|_e| ArgParseErr::with_msg("invalid unsharp geometry"))?;
        let parts: Vec<&str> = string.split('x').collect();

        match parts.len() {
            // we don't have a sigma, but do have a radius and maybe gain and maybe threshold
            1 => {
                let subparts: Vec<&str> = string.split('+').collect();
                let radius = parse_radius(subparts.first().unwrap())?;
                Ok(parse_rest(radius, Some(Default::default()), subparts)?)
            }
            // we do have a radius and sigma, and maybe gain and maybe threshold
            2 => {
                let radius = parse_radius(parts.first().unwrap())?;
                let subparts: Vec<&str> = parts.get(1).unwrap().split('+').collect();
                Ok(parse_rest(radius, None, subparts)?)
            }
            _ => Err(ArgParseErr::with_msg("invalid unsharp geometry format")),
        }
    }
}

fn parse_radius(s: &str) -> Result<usize, ArgParseErr> {
    strip_and_parse_number::<usize>(s).map_err(|_| ArgParseErr::with_msg("invalid radius value"))
}

fn parse_sigma(s: &str) -> Result<f32, ArgParseErr> {
    strip_and_parse_number::<f32>(s).map_err(|_| ArgParseErr::with_msg("invalid sigma value"))
}

fn parse_gain(s: &str) -> Result<f32, ArgParseErr> {
    strip_and_parse_number::<f32>(s).map_err(|_| ArgParseErr::with_msg("invalid gain value"))
}

fn parse_threshold(s: &str) -> Result<i32, ArgParseErr> {
    strip_and_parse_number::<i32>(s).map_err(|_| ArgParseErr::with_msg("invalid threshold value"))
}

fn parse_rest(
    radius: usize,
    sigma: Option<f32>,
    rest: Vec<&str>,
) -> Result<UnsharpenGeometry, ArgParseErr> {
    match rest.len() {
        1 => Ok(UnsharpenGeometry {
            radius,
            sigma: sigma.map_or_else(|| parse_sigma(rest.first().unwrap()), |v| Ok(v))?,
            ..Default::default()
        }),
        2 => Ok(UnsharpenGeometry {
            radius,
            sigma: sigma.map_or_else(|| parse_sigma(rest.first().unwrap()), |v| Ok(v))?,
            gain: parse_gain(rest.get(1).unwrap())?,
            ..Default::default()
        }),
        3 => Ok(UnsharpenGeometry {
            radius,
            sigma: sigma.map_or_else(|| parse_sigma(rest.first().unwrap()), |v| Ok(v))?,
            gain: parse_gain(rest.get(1).unwrap())?,
            threshold: parse_threshold(rest.get(2).unwrap())?,
            ..Default::default()
        }),
        _ => Err(ArgParseErr::with_msg("invalid unsharp geometry format")),
    }
}

#[cfg(test)]
mod tests {
    use super::UnsharpenGeometry;
    use std::str::FromStr;

    #[test]
    fn test_radius_only() {
        assert_eq!(
            UnsharpenGeometry::from_str("5"),
            Ok(UnsharpenGeometry {
                radius: 5,
                ..Default::default()
            })
        );
    }

    #[test]
    fn test_radius_and_sigma() {
        assert_eq!(
            UnsharpenGeometry::from_str("5x1.1"),
            Ok(UnsharpenGeometry {
                radius: 5,
                sigma: 1.1,
                ..Default::default()
            })
        );
    }

    #[test]
    fn test_radius_and_sigma_and_gain() {
        assert_eq!(
            UnsharpenGeometry::from_str("2x1.5+9"),
            Ok(UnsharpenGeometry {
                radius: 2,
                sigma: 1.5,
                gain: 9.0,
                ..Default::default()
            })
        );
    }

    #[test]
    fn test_radius_and_sigma_and_gain_and_threshold() {
        assert_eq!(
            UnsharpenGeometry::from_str("42x2.1+7+11"),
            Ok(UnsharpenGeometry {
                radius: 42,
                sigma: 2.1,
                gain: 7.0,
                threshold: 11,
            })
        );
    }

    #[test]
    fn test_radius_and_gain_and_threshold() {
        assert_eq!(
            UnsharpenGeometry::from_str("42+7+11"),
            Ok(UnsharpenGeometry {
                radius: 42,
                gain: 7.0,
                threshold: 11,
                ..Default::default()
            })
        );
    }

    #[test]
    fn test_invalid() {
        assert!(UnsharpenGeometry::from_str("💥 not ascii only").is_err());
        assert!(UnsharpenGeometry::from_str("").is_err());
        assert!(UnsharpenGeometry::from_str("abc").is_err());
        assert!(UnsharpenGeometry::from_str("0x0x0x0x").is_err());
    }
}
