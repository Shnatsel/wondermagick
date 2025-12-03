use crate::arg_parse_err::ArgParseErr;
use std::ffi::OsStr;

#[derive(Debug, Clone, PartialEq, strum::Display, strum::EnumString, strum::IntoStaticStr)]
#[strum(ascii_case_insensitive)]
pub enum GrayscaleMethod {
    Rec601Luma,
    Rec601Luminance,
    Rec709Luma,
    Rec709Luminance,
    Brightness,
    Lightness,
}

impl TryFrom<&std::ffi::OsStr> for GrayscaleMethod {
    type Error = ArgParseErr;

    fn try_from(s: &OsStr) -> Result<Self, Self::Error> {
        if let Some(s_utf8) = s.to_str() {
            if let Ok(known_method) = Self::try_from(s_utf8) {
                return Ok(known_method);
            }
        }
        Err(ArgParseErr::with_msg(format!(
            "unrecognized grayscale method {}",
            s.to_string_lossy()
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::GrayscaleMethod;
    use std::str::FromStr;

    #[test]
    fn test_valid_methods() {
        assert_eq!(
            GrayscaleMethod::from_str("Rec709Luma"),
            Ok(GrayscaleMethod::Rec709Luma)
        );
    }

    #[test]
    fn test_case_insensitive() {
        assert_eq!(
            GrayscaleMethod::from_str("rec709luminance"),
            Ok(GrayscaleMethod::Rec709Luminance)
        );
    }

    #[test]
    fn test_invalid() {
        assert!(GrayscaleMethod::from_str("💥 non-asccii").is_err());
        assert!(GrayscaleMethod::from_str("").is_err());
        assert!(GrayscaleMethod::from_str("foo").is_err());
    }
}
