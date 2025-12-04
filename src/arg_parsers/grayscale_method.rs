use crate::arg_parse_err::ArgParseErr;
use std::ffi::OsStr;

/// https://imagemagick.org/script/command-line-options.php#intensity
/// Running ImageMagick 6's `convert -list intensity` shows all available methods.
#[derive(Debug, Clone, PartialEq, strum::Display, strum::EnumString, strum::IntoStaticStr)]
#[strum(ascii_case_insensitive)]
pub enum GrayscaleMethod {
    Average,
    Brightness,
    Lightness,
    MS,
    Mean,
    RMS,
    Rec601Luma,
    Rec601Luminance,
    Rec709Luma,
    Rec709Luminance,
}

impl TryFrom<&std::ffi::OsStr> for GrayscaleMethod {
    type Error = ArgParseErr;

    fn try_from(s: &OsStr) -> Result<Self, Self::Error> {
        s.to_str()
            .ok_or_else(|| ArgParseErr::with_msg("non-utf8 grayscale value"))
            .and_then(|val| Self::try_from(val).map_err(ArgParseErr::with_msg))
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
