use crate::{arg_parse_err::ArgParseErr, arg_parsers::strip_and_parse_number};
use std::{ffi::OsStr, str::FromStr};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct BlurGeometry {
    radius: usize, // only to match imagemagick's cli behaviour, acutal value is ignored
    pub sigma: f32,
}

impl FromStr for BlurGeometry {
    type Err = ArgParseErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(OsStr::new(s))
    }
}

impl TryFrom<&OsStr> for BlurGeometry {
    type Error = ArgParseErr;

    fn try_from(s: &OsStr) -> Result<Self, Self::Error> {
        if s.is_empty() {
            return Err(ArgParseErr::with_msg(
                "blur geometry must be non-empty and ASCII only",
            ));
        }

        let string: &str = s
            .try_into()
            .map_err(|_e| ArgParseErr::with_msg("invalid blur geometry"))?;
        let parts: Vec<&str> = string.split('x').collect();

        match parts.len() {
            1 => parse_radius(string).map(|radius| Self {
                radius,
                sigma: 1.2, // no science behind the value, just eyeballed to match ImageMagick's default
            }),
            2 => {
                match (
                    parse_radius(parts.first().unwrap()),
                    parse_sigma(parts.get(1).unwrap()),
                ) {
                    (Ok(radius), Ok(sigma)) => Ok(Self { radius, sigma }),
                    (Err(_), Ok(sigma)) => Ok(Self {
                        sigma,
                        ..Default::default()
                    }),
                    _ => Err(ArgParseErr::with_msg("invalid sigma value")),
                }
            }
            _ => Err(ArgParseErr::with_msg("invalid blur geometry format")),
        }
    }
}

fn parse_radius(s: &str) -> Result<usize, ArgParseErr> {
    strip_and_parse_number::<usize>(s).map_err(|_| ArgParseErr::with_msg("invalid radius value"))
}

fn parse_sigma(s: &str) -> Result<f32, ArgParseErr> {
    strip_and_parse_number::<f32>(s).map_err(|_| ArgParseErr::with_msg("invalid sigma value"))
}

#[cfg(test)]
mod tests {
    use super::BlurGeometry;
    use std::str::FromStr;

    #[test]
    fn test_radius_only() {
        let geom = BlurGeometry::from_str("5").unwrap();
        assert_eq!(geom.radius, 5);
    }

    #[test]
    fn test_sigma_only() {
        let geom = BlurGeometry::from_str("x1337").unwrap();
        assert_eq!(geom.radius, 0);
        assert_eq!(geom.sigma, 1337.0);
    }

    #[test]
    fn test_radius_and_sigma_decimal() {
        let geom = BlurGeometry::from_str("5x1.0").unwrap();
        assert_eq!(geom.radius, 5);
        assert_eq!(geom.sigma, 1.0);
    }

    #[test]
    fn test_radius_and_sigma_int() {
        let geom = BlurGeometry::from_str("5x1").unwrap();
        assert_eq!(geom.radius, 5);
        assert_eq!(geom.sigma, 1.0);
    }

    #[test]
    fn test_invalid() {
        assert!(BlurGeometry::from_str("💥 not ascii only").is_err());
        assert!(BlurGeometry::from_str("").is_err());
        assert!(BlurGeometry::from_str("abc").is_err());
    }
}
