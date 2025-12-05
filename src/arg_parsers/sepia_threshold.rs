use crate::arg_parse_err::ArgParseErr;
use std::{ffi::OsStr, str::FromStr};

#[derive(Debug, Clone, PartialEq)]
pub struct SepiaThreshold(pub f32);

impl FromStr for SepiaThreshold {
    type Err = ArgParseErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(OsStr::new(s))
    }
}

impl TryFrom<&OsStr> for SepiaThreshold {
    type Error = ArgParseErr;

    fn try_from(s: &OsStr) -> Result<Self, Self::Error> {
        match s.to_str() {
            None => Err(ArgParseErr::with_msg("non-utf8 sepia threshold value")),
            Some(s) => {
                if s.ends_with("%") && s.len() > 1 {
                    s[0..(s.len() - 1)]
                        .parse::<f32>()
                        .map_err(|_| ArgParseErr::with_msg("invalid sepia threshold format"))
                        .map(|v| SepiaThreshold(v / 100.0))
                } else {
                    s.parse::<f32>()
                        .map_err(|_| ArgParseErr::with_msg("invalid sepia threshold format"))
                        .map(SepiaThreshold)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SepiaThreshold;
    use std::str::FromStr;

    #[test]
    fn test_percentages() {
        assert_eq!(SepiaThreshold::from_str("80%"), Ok(SepiaThreshold(0.8)));
        assert_eq!(SepiaThreshold::from_str("5%"), Ok(SepiaThreshold(0.05)));
        assert_eq!(SepiaThreshold::from_str("05%"), Ok(SepiaThreshold(0.05)));
        assert_eq!(
            SepiaThreshold::from_str("99.9999%"),
            Ok(SepiaThreshold(0.999999))
        );
    }

    #[test]
    fn test_floats() {
        assert_eq!(SepiaThreshold::from_str("0.8"), Ok(SepiaThreshold(0.8)));
        assert_eq!(SepiaThreshold::from_str("100.8"), Ok(SepiaThreshold(100.8)));
    }

    #[test]
    fn test_invalid() {
        assert!(SepiaThreshold::from_str("0.8%%").is_err());
        assert!(SepiaThreshold::from_str("%").is_err());
        assert!(SepiaThreshold::from_str("abc%").is_err());
    }
}
