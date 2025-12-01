use crate::arg_parse_err::ArgParseErr;
use std::{ffi::OsStr, str::FromStr};

#[derive(Debug, Clone, PartialEq)]
pub enum GrayscaleMethod {
    Rec601Luma,
    Rec601Luminance,
    Rec709Luma,
    Rec709Luminance,
    Brightness,
    Lightness,
}

impl FromStr for GrayscaleMethod {
    type Err = ArgParseErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(OsStr::new(s))
    }
}

impl TryFrom<&std::ffi::OsStr> for GrayscaleMethod {
    type Error = ArgParseErr;

    fn try_from(s: &std::ffi::OsStr) -> Result<Self, Self::Error> {
        let string: &str = s
            .to_str()
            .ok_or_else(|| ArgParseErr::with_msg("invalid grayscale method"))?;
        match string.to_lowercase().as_str() {
            "Rec601Luma" => Ok(GrayscaleMethod::Rec601Luma),
            "Rec601Luminance" => Ok(GrayscaleMethod::Rec601Luminance),
            "Rec709Luma" => Ok(GrayscaleMethod::Rec709Luma),
            "Rec709Luminance" => Ok(GrayscaleMethod::Rec709Luminance),
            "Brightness" => Ok(GrayscaleMethod::Brightness),
            "Lightness" => Ok(GrayscaleMethod::Lightness),
            _ => Err(ArgParseErr::with_msg("unknown grayscale method")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::GrayscaleMethod;
    use std::str::FromStr;

    #[test]
    fn test_invalid() {
        assert!(GrayscaleMethod::from_str("💥 non-asccii").is_err());
        assert!(GrayscaleMethod::from_str("").is_err());
        assert!(GrayscaleMethod::from_str("foo").is_err());
    }
}
