use crate::arg_parse_err::ArgParseErr;
use std::{ffi::OsStr, str::FromStr};

#[derive(Debug, Clone, PartialEq)]
pub enum Gravity {
    NorthWest,
    North,
    NorthEast,
    West,
    Center,
    East,
    SouthWest,
    South,
    SouthEast,
}

impl FromStr for Gravity {
    type Err = ArgParseErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(OsStr::new(s))
    }
}

impl TryFrom<&OsStr> for Gravity {
    type Error = ArgParseErr;

    fn try_from(s: &OsStr) -> Result<Self, Self::Error> {
        if s.is_empty() {
            return Err(ArgParseErr::with_msg("gravity type must be non-empty"));
        }

        let string: &str = s
            .try_into()
            .map_err(|_e| ArgParseErr::with_msg("invalid gravity type"))?;

        match string.to_lowercase().as_str() {
            "center" => Ok(Gravity::Center),
            "north" => Ok(Gravity::North),
            "south" => Ok(Gravity::South),
            "east" => Ok(Gravity::East),
            "west" => Ok(Gravity::West),
            "northeast" => Ok(Gravity::NorthEast),
            "northwest" => Ok(Gravity::NorthWest),
            "southeast" => Ok(Gravity::SouthEast),
            "southwest" => Ok(Gravity::SouthWest),
            _ => Err(ArgParseErr::with_msg("invalid gravity argument")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Gravity;
    use std::str::FromStr;

    #[test]
    fn test_case_insensitive() {
        assert_eq!(Gravity::from_str("NorthWest"), Ok(Gravity::NorthWest));
        assert_eq!(Gravity::from_str("northwest"), Ok(Gravity::NorthWest));
    }

    #[test]
    fn test_invalid() {
        assert!(Gravity::from_str("unknown").is_err());
        assert!(Gravity::from_str("💥 non-ascii").is_err());
        assert!(Gravity::from_str("").is_err());
    }
}
