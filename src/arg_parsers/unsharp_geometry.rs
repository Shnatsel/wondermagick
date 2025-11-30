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
                "unsharp geometry must be non-empty and ASCII only",
            ));
        }

        let string: &str = s
            .try_into()
            .map_err(|_e| ArgParseErr::with_msg("invalid unsharp geometry"))?;
        let parts: Vec<&str> = string.split('x').collect();
        println!("parts: {:?}", parts);

        match parts.len() {
            1 => {
                // we don't have a sigma, but must have a radius and maybe gain and maybe threshold
                let subparts: Vec<&str> = string.split('+').collect();

                match subparts.len() {
                    1 => {
                        let radius = strip_and_parse_number::<usize>(string)
                            .map_err(|_| ArgParseErr::with_msg("invalid radius value"))?;
                        return Ok(Self {
                            radius,
                            ..Default::default()
                        });
                    }
                    2 => {
                        let radius = strip_and_parse_number::<usize>(subparts.first().unwrap())
                            .map_err(|_| ArgParseErr::with_msg("invalid radius value"))?;
                        let gain = strip_and_parse_number::<f32>(subparts.get(1).unwrap())
                            .map_err(|_| ArgParseErr::with_msg("invalid gain value"))?;
                        return Ok(Self {
                            radius,
                            gain,
                            ..Default::default()
                        });
                    }
                    3 => {
                        let radius = strip_and_parse_number::<usize>(subparts.first().unwrap())
                            .map_err(|_| ArgParseErr::with_msg("invalid radius value"))?;
                        let gain = strip_and_parse_number::<f32>(subparts.get(1).unwrap())
                            .map_err(|_| ArgParseErr::with_msg("invalid gain value"))?;
                        let threshold = strip_and_parse_number::<i32>(subparts.get(2).unwrap())
                            .map_err(|_| ArgParseErr::with_msg("invalid threshold value"))?;
                        return Ok(Self {
                            radius,
                            gain,
                            threshold,
                            ..Default::default()
                        });
                    }
                    _ => Err(ArgParseErr::with_msg("invalid unsharp geometry format")),
                }
            }
            2 => {
                let radius =
                    strip_and_parse_number::<usize>(parts.first().unwrap()).map_err(|_| {
                        ArgParseErr::with_msg("invalid radius value in unsharp geometry")
                    })?;
                let subparts: Vec<&str> = string.split('+').collect();

                match subparts.len() {
                    1 => {
                        let sigma =
                            strip_and_parse_number::<f32>(parts.get(1).unwrap()).map_err(|_| {
                                ArgParseErr::with_msg("invalid sigma value in unsharp geometry")
                            })?;
                        return Ok(Self {
                            radius,
                            sigma,
                            ..Default::default()
                        });
                    }
                    2 => {
                        let sigma = strip_and_parse_number::<f32>(subparts.first().unwrap())
                            .map_err(|_| {
                                ArgParseErr::with_msg("invalid sigma value in unsharp geometry")
                            })?;
                        let gain = strip_and_parse_number::<f32>(subparts.get(1).unwrap())
                            .map_err(|_| ArgParseErr::with_msg("invalid gain value"))?;
                        return Ok(Self {
                            radius,
                            sigma,
                            gain,
                            ..Default::default()
                        });
                    }
                    3 => {
                        let gain = strip_and_parse_number::<f32>(subparts.get(1).unwrap())
                            .map_err(|_| ArgParseErr::with_msg("invalid gain value"))?;
                        let threshold = strip_and_parse_number::<i32>(subparts.get(2).unwrap())
                            .map_err(|_| ArgParseErr::with_msg("invalid threshold value"))?;
                        return Ok(Self {
                            radius,
                            gain,
                            threshold,
                            ..Default::default()
                        });
                    }
                    _ => Err(ArgParseErr::with_msg("invalid unsharp geometry format")),
                }
            }
            _ => Err(ArgParseErr::with_msg("invalid unsharp geometry format")),
        }
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
    fn test_radius_and_gain_and_threshold() {
        let geom = UnsharpGeometry::from_str("42+7+11").unwrap();
        assert_eq!(
            geom,
            UnsharpGeometry {
                radius: 42,
                gain: 7.0,
                threshold: 11,
                ..Default::default()
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
