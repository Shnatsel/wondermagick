// Fun fact: the geometry documentation at https://www.imagemagick.org/Magick++/Geometry.html is a lie.
//
// It says things like
// > Offsets must be given as pairs; in other words, in order to specify either xoffset or yoffset both must be present.
// but this works:
// `convert rose: -crop 50x+0 crop_half.gif`
//
// Maybe it's an extended geometry feature, or maybe it just always works that way and documentation is a lie!

use std::ffi::OsStr;
use std::fmt::Display;
use std::str::{self, FromStr};

use crate::arg_parse_err::ArgParseErr;

#[cfg(test)]
use crate::utils::arbitrary;
#[cfg(test)]
use quickcheck::Arbitrary;

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct Geometry {
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub xoffset: Option<f64>,
    pub yoffset: Option<f64>,
}

#[cfg(test)]
impl Arbitrary for Geometry {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        Self {
            width: arbitrary::optional_positive_float(g),
            height: arbitrary::optional_positive_float(g),
            xoffset: arbitrary::optional_nonzero_float(g),
            yoffset: arbitrary::optional_nonzero_float(g),
        }
    }
}

impl Display for Geometry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // fully empty geometry is 'x'
        if [self.width, self.height, self.xoffset, self.yoffset]
            .iter()
            .all(|v| v.is_none())
        {
            return write!(f, "x");
        }

        if let Some(w) = self.width {
            write!(f, "{w}")?;
        }
        if self.height.is_some() || self.xoffset.is_some() || self.yoffset.is_some() {
            write!(f, "x")?;
        }
        if let Some(h) = self.height {
            write!(f, "{h}")?;
        }
        match (self.xoffset, self.yoffset) {
            (Some(x), Some(y)) => write!(f, "{x:+}{y:+}"),
            (Some(x), None) => write!(f, "{x:+}"),
            (None, Some(y)) => write!(f, "+0{y:+}"),
            (None, None) => Ok(()),
        }
    }
}

impl FromStr for Geometry {
    type Err = ArgParseErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(OsStr::new(s))
    }
}

impl TryFrom<&OsStr> for Geometry {
    type Error = ArgParseErr;

    fn try_from(s: &OsStr) -> Result<Self, Self::Error> {
        if !s.is_ascii() {
            return Err(ArgParseErr::new());
        }

        let mut ascii = s.as_encoded_bytes();
        let mut result = Geometry::default();

        if ascii.len() == 0 {
            // emptry string is an error in -resize, -crop and crop-on-load in `convert`
            return Err(ArgParseErr::new());
        }

        if let Some(next_char) = ascii.first() {
            if ![b'x', b'+', b'-'].contains(next_char) {
                result.width = Some(read_positive_float(&mut ascii).ok_or(ArgParseErr::new())?);
            }
        }
        if let Some(next_char) = ascii.first() {
            if next_char == &b'x' {
                ascii = &ascii[1..]; // skip the 'x'

                if let Some(next_char) = ascii.first() {
                    // width cannot be signed, if there's a sign it's an offset
                    if ![b'+', b'-'].contains(next_char) {
                        result.height =
                            Some(read_positive_float(&mut ascii).ok_or(ArgParseErr::new())?);
                    }
                }

                // We try to read signed offsets afterwards ONLY if there was an 'x' to mimic imagemagick
                if let Some(next_char) = ascii.first() {
                    if [b'+', b'-'].contains(next_char) {
                        let offset = read_signed_float(&mut ascii).ok_or(ArgParseErr::new())?;
                        result.xoffset = Some(offset);
                    }
                }
                if let Some(next_char) = ascii.first() {
                    if [b'+', b'-'].contains(next_char) {
                        let offset = read_signed_float(&mut ascii).ok_or(ArgParseErr::new())?;
                        result.yoffset = Some(offset);
                    }
                }
            }
        }

        if !ascii.is_empty() {
            // there was some unexpected content at the end we didn't parse
            Err(ArgParseErr::new())
        } else {
            Ok(result)
        }
    }
}

fn read_positive_float(input: &mut &[u8]) -> Option<f64> {
    read_float(input, false)
}

fn read_signed_float(input: &mut &[u8]) -> Option<f64> {
    read_float(input, true)
}

fn read_float(input: &mut &[u8], allow_sign: bool) -> Option<f64> {
    let mut count = 0;
    if [Some(&b'+'), Some(&b'-')].contains(&input.first()) {
        match allow_sign {
            true => count += 1,
            false => return None,
        }
    }
    count += count_leading_digits(&input[count..]);
    if input.get(count) == Some(&b'.') {
        // imagemagick permits having a trailing dot with no digits following it
        count += 1;
        count += count_leading_digits(&input[count..]);
    }

    let (number, remainder) = input.split_at(count);
    let float = str::from_utf8(number).unwrap().parse::<f64>().unwrap();
    *input = remainder;
    Some(float)
}

fn count_leading_digits(input: &[u8]) -> usize {
    input
        .iter()
        .copied()
        .take_while(|b| b.is_ascii_digit())
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    use quickcheck_macros::quickcheck;

    #[test]
    fn test_full_positive_geometry() {
        let expected = Geometry {
            width: Some(5.0),
            height: Some(10.0),
            xoffset: Some(15.0),
            yoffset: Some(20.0),
        };
        let parsed = Geometry::from_str("5x10+15+20").unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_missing_height() {
        // Needed for resize and crop (extended) geometry parsing
        let expected = Geometry {
            width: Some(5.0),
            height: None,
            xoffset: Some(15.0),
            yoffset: Some(20.0),
        };
        let parsed = Geometry::from_str("5x+15+20").unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_missing_height_invalid() {
        // resize command rejects this
        assert!(Geometry::from_str("5+15+20").is_err());
    }

    #[test]
    fn test_negative_yoffset() {
        let orig = Geometry {
            width: Some(1.0),
            height: Some(2.404677232457912e106),
            xoffset: Some(-22.454616716202054),
            yoffset: Some(-3.938307476584102e33),
        };
        let stringified = orig.to_string();
        let parsed = Geometry::from_str(&stringified).unwrap();
        assert_eq!(orig, parsed)
    }

    #[quickcheck]
    fn roundtrip_is_lossless(orig: Geometry) {
        let stringified = orig.to_string();
        let mut parsed = Geometry::from_str(&stringified).unwrap();

        // In order to express some sort of y offset we need to specify a zero x offset,
        // so roundtrip cannot be entirely lossless.
        // We special-case that here.
        if orig.yoffset.is_some() {
            if orig.xoffset == Some(0.0) || orig.xoffset == None {
                parsed.xoffset = orig.xoffset;
            }
        }

        assert_eq!(orig, parsed)
    }
}
