use crate::arg_parse_err::ArgParseErr;
use std::ffi::OsStr;

#[derive(Debug, Clone, PartialEq, strum::Display, strum::EnumString, strum::IntoStaticStr)]
#[strum(ascii_case_insensitive)]
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
    None,
    Forget,
}

impl TryFrom<&OsStr> for Gravity {
    type Error = ArgParseErr;

    fn try_from(s: &OsStr) -> Result<Self, Self::Error> {
        s.to_str()
            .ok_or_else(|| ArgParseErr::with_msg("non-utf8 gravity value"))
            .and_then(|val| Gravity::try_from(val).map_err(ArgParseErr::with_msg))
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
