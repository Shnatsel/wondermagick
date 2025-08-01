//! Parser for rotate argument.

use std::ffi::OsStr;

use crate::arg_parse_err::ArgParseErr;

/// Parsed rotation specification.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct RotateGeometry {
    pub degrees: f64,
    /// Rotate only if width > height.
    pub only_if_wider: bool,
    /// Rotate only if width < height.
    pub only_if_taller: bool,
}

impl TryFrom<&OsStr> for RotateGeometry {
    type Error = ArgParseErr;

    fn try_from(value: &OsStr) -> Result<Self, Self::Error> {
        let s = value.to_str().ok_or_else(|| ArgParseErr::new())?;

        // Split at where the degree numeric ends.
        let mut it = s.split_inclusive(|c: char| c != '-' && !c.is_digit(10));

        let degrees_str = it.next().expect("the iterator will have at least one item");
        let modifiers = it.flat_map(|c| c.chars());

        // Parse the degrees.
        let degrees = degrees_str
            .trim()
            .parse::<f64>()
            .map_err(|_| ArgParseErr::new())?;

        // Parse modifiers.
        let mut only_if_wider = false;
        let mut only_if_taller = false;

        for modifier in modifiers {
            let flag = match modifier {
                '>' => &mut only_if_wider,
                '<' => &mut only_if_taller,
                _ => return Err(ArgParseErr::new()),
            };
            if *flag {
                return Err(ArgParseErr::new());
            }
            *flag = true;
        }

        Ok(RotateGeometry {
            degrees,
            only_if_wider,
            only_if_taller,
        })
    }
}
