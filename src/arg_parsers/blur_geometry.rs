use crate::{arg_parse_err::ArgParseErr, arg_parsers::strip_and_parse_number};
use std::{ffi::OsStr, str::FromStr};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct BlurGeometry {
    radius: usize, // only to match imagemagick's cli behaviour, acutal value is ignored
    pub sigma: Sigma,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Sigma(pub f32);

impl Default for Sigma {
    fn default() -> Self {
        Sigma(5.0) // no science behind the value, just eyeballed to match imagemagick's default
    }
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
        if !s.is_ascii() || s.is_empty() {
            return Err(ArgParseErr::new());
        }

        let string: &str = s.try_into().map_err(|_e| ArgParseErr::new())?;
        let parts: Vec<&str> = string.split('x').collect();

        match parts.len() {
            1 => strip_and_parse_number::<usize>(parts.first().unwrap())
                .map_err(|_| ArgParseErr::new())
                .map(|radius: usize| Self {
                    radius,
                    ..Default::default()
                }),
            2 => {
                let maybe_radius = strip_and_parse_number::<usize>(parts.first().unwrap());
                let maybe_sigma = strip_and_parse_number::<f32>(parts.get(1).unwrap());
                match (maybe_radius, maybe_sigma) {
                    (Ok(radius), Ok(sigma)) => Ok(Self {
                        radius,
                        sigma: Sigma(sigma),
                    }),
                    (Err(_), Ok(sigma)) => Ok(Self {
                        sigma: Sigma(sigma),
                        ..Default::default()
                    }),
                    _ => Err(ArgParseErr::new()),
                }
            }
            _ => Err(ArgParseErr::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{BlurGeometry, Sigma};
    use std::str::FromStr;

    #[test]
    fn test_default() {
        let geom = BlurGeometry::default();
        assert_eq!(geom.radius, 0);
        assert_eq!(geom.sigma, Sigma(5.0));
    }

    #[test]
    fn test_radius_only() {
        let geom = BlurGeometry::from_str("5").unwrap();
        assert_eq!(geom.radius, 5);
        assert_eq!(geom.sigma, Sigma::default());
    }

    #[test]
    fn test_sigma_only() {
        let geom = BlurGeometry::from_str("x1337").unwrap();
        assert_eq!(geom.radius, 0);
        assert_eq!(geom.sigma, Sigma(1337.0));
    }

    #[test]
    fn test_radius_and_sigma_decimal() {
        let geom = BlurGeometry::from_str("5x1.0").unwrap();
        assert_eq!(geom.radius, 5);
        assert_eq!(geom.sigma, Sigma(1.0));
    }

    #[test]
    fn test_radius_and_sigma_int() {
        let geom = BlurGeometry::from_str("5x1").unwrap();
        assert_eq!(geom.radius, 5);
        assert_eq!(geom.sigma, Sigma(1.0));
    }

    #[test]
    fn test_invalid() {
        assert!(BlurGeometry::from_str("💥 not ascii only").is_err());
        assert!(BlurGeometry::from_str("").is_err());
        assert!(BlurGeometry::from_str("abc").is_err());
    }
}
