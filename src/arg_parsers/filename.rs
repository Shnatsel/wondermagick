use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    str::FromStr,
};

// For .as_bytes() and .from_bytes() on &OsStr
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
#[cfg(target_os = "wasi")]
use std::os::wasi::ffi::OsStrExt;
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;

use crate::{error::MagickError, wm_err};

use super::{Geometry, ResizeGeometry};

#[derive(Debug, Clone, PartialEq)]
pub struct InputFileArg {
    pub path: PathBuf,
    //format: Option<String>, // TODO: turn into an enum and enable
    pub read_mod: Option<ReadModifier>,
}

impl InputFileArg {
    pub fn parse(input: &OsStr) -> Result<Self, MagickError> {
        if file_exists(input) {
            Ok(Self {
                path: PathBuf::from(input),
                read_mod: None,
            })
        } else {
            let parse_result = split_off_bracketed_suffix(input).and_then(|(path, modifier)| {
                if let Ok(valid_modifier) = ReadModifier::try_from(modifier) {
                    Some((path, valid_modifier))
                } else {
                    None
                }
            });
            if let Some((path, modifier)) = parse_result {
                if file_exists(path) {
                    Ok(Self {
                        path: PathBuf::from(input),
                        read_mod: Some(modifier),
                    })
                } else {
                    Err(wm_err!(
                        "unable to open image `{}': No such file or directory",
                        path.display()
                    )) // TODO: more accurate error reporting
                }
            } else {
                // the file does not exist and there is no valid modifier on it
                Err(wm_err!(
                    "unable to open image `{}': No such file or directory",
                    input.display()
                )) // TODO: more accurate error reporting
            }
        }
    }
}

/// The action to be taken upon loading the image.
/// `convert` accepts any single one of: frame selection, resize, or crop.
///
/// See <https://imagemagick.org/Usage/files/#read_mods> for details.
/// I've also verified it behaves according to the documentation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReadModifier {
    Resize(ResizeGeometry),
    Crop(LoadCropGeometry),
    // TODO: frame selection.
}

impl FromStr for ReadModifier {
    type Err = MagickError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(OsStr::new(s))
    }
}

impl TryFrom<&OsStr> for ReadModifier {
    type Error = MagickError;

    fn try_from(s: &OsStr) -> Result<Self, Self::Error> {
        if !s.is_ascii() {
            return Err(wm_err!("invalid read modifier: {}", s.to_string_lossy()));
        }

        let ascii = s.as_encoded_bytes();
        let x_count = ascii.iter().copied().filter(|c| *c == b'x').take(2).count();
        let plus_count = ascii.iter().copied().filter(|c| *c == b'+').take(3).count();

        if x_count == 1 && plus_count == 0 {
            Ok(Self::Resize(ResizeGeometry::try_from(s)?))
        } else if x_count == 1 && plus_count == 2 {
            Ok(Self::Crop(LoadCropGeometry::try_from(s)?))
        } else {
            Err(wm_err!("frame selection is not yet supported")) // TODO: support [0] at least
        }
    }
}

/// On loading only a subset of crop geometry specification is supported:
/// it *must* be in the form AxB+C+D, see
/// https://imagemagick.org/Usage/files/#read_mods
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LoadCropGeometry {
    pub width: u32,
    pub height: u32,
    pub xoffset: u32,
    pub yoffset: u32,
}

impl FromStr for LoadCropGeometry {
    type Err = MagickError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(OsStr::new(s))
    }
}

impl TryFrom<&OsStr> for LoadCropGeometry {
    type Error = MagickError;

    fn try_from(s: &OsStr) -> Result<Self, Self::Error> {
        let geom = Geometry::try_from(s)?;

        let convert_field = |field: Option<f64>| -> Result<u32, MagickError> {
            let f =
                field.ok_or_else(|| wm_err!("invalid crop geometry: {}", s.to_string_lossy()))?;
            if f.is_sign_negative() {
                Err(wm_err!("invalid crop geometry: {}", s.to_string_lossy()))
            } else {
                Ok(f as u32)
            }
        };

        Ok(Self {
            width: convert_field(geom.width)?,
            height: convert_field(geom.height)?,
            xoffset: convert_field(geom.xoffset)?,
            yoffset: convert_field(geom.yoffset)?,
        })
    }
}

fn file_exists(path: &OsStr) -> bool {
    let path = Path::new(path);
    path.is_file()
}

fn split_off_bracketed_suffix(input: &OsStr) -> Option<(&OsStr, &OsStr)> {
    let bytes = input.as_bytes();

    if bytes.is_empty() || bytes.last() != Some(&b']') {
        return None;
    }

    // Search for the last '[' before the final ']'.
    // bytes.len() - 1 is the index of the final ']'
    // So we search in the slice bytes[0 .. bytes.len() - 1]
    let slice_before_closing_bracket = &bytes[0..bytes.len() - 1];

    match slice_before_closing_bracket
        .iter()
        .rposition(|&b| b == b'[')
    {
        Some(open_bracket_idx) => {
            // `open_bracket_idx` is the index of '[' in the original `bytes` slice.

            // Part before the '['
            let prefix_bytes = &bytes[0..open_bracket_idx];

            // Part between '[' and ']'
            // It starts one after open_bracket_idx and ends one before the closing ']'
            // (which is at bytes.len() - 1)
            let inner_content_bytes = &bytes[open_bracket_idx + 1..bytes.len() - 1];

            // OsStr is guaranteed to be a superset of UTF-8 and of 7-bit ascii.
            // Slicing by ASCII characters `[` and `]` means that if the original `OsStr`
            // was valid, these byte sub-slices will also form valid `OsStr`s.
            // OsStr::from_bytes is the counterpart to as_bytes.
            let prefix_os_str = OsStr::from_bytes(prefix_bytes);
            let inner_content_os_str = OsStr::from_bytes(inner_content_bytes);

            Some((prefix_os_str, inner_content_os_str))
        }
        None => {
            // Found ']' at the end, but no matching '[' before it.
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn load_crop_geometry() {
        // only a basic smoke test because the underlying geometry parser is well tested already
        let expected = LoadCropGeometry {
            width: 1,
            height: 2,
            xoffset: 3,
            yoffset: 4,
        };
        let parsed = LoadCropGeometry::from_str("1x2+3+4").unwrap();
        assert_eq!(expected, parsed);
    }

    #[test]
    fn load_crop_read_modifier() {
        // only a basic smoke test because the underlying geometry parser is well tested already
        let expected = ReadModifier::Crop(LoadCropGeometry::from_str("1x2+3+4").unwrap());
        let parsed = ReadModifier::from_str("1x2+3+4").unwrap();
        assert_eq!(expected, parsed);
    }

    #[test]
    fn load_resize_read_modifier() {
        // only a basic smoke test because the underlying geometry parser is well tested already
        let expected = ReadModifier::Resize(ResizeGeometry::from_str("40x60").unwrap());
        let parsed = ReadModifier::from_str("40x60").unwrap();
        assert_eq!(expected, parsed);
    }

    #[test]
    fn test_split_off_bracketed_suffix() {
        // These are fairly basic tests, they weren't validated against imagemagick behavior,
        // but it shouldn't really matter since imagemagick only accepts valid geometry or frame specs anyway
        let test_cases = vec![
            ("filename[metadata]", Some(("filename", "metadata"))),
            ("file[v1][v2]", Some(("file[v1]", "v2"))),
            ("nodata]", None),
            ("nodata", None),
            ("[onlydata]", Some(("", "onlydata"))),
            ("data[]", Some(("data", ""))),
            ("[]", Some(("", ""))),
            ("abc[def]ghi", None),
            ("abc[d[e]f]", Some(("abc[d", "e]f"))),
            ("", None),
            ("]", None),
            ("[", None),
            ("test[ ]", Some(("test", " "))),
            ("test[[nested]]", Some(("test[", "nested]"))),
        ];

        for (input_str, expected_output) in test_cases {
            let input_os_str = OsStr::new(input_str);
            let result = split_off_bracketed_suffix(input_os_str);

            match (result, expected_output) {
                (Some((res_prefix, res_suffix)), Some((exp_prefix, exp_suffix))) => {
                    assert_eq!(
                        res_prefix,
                        OsStr::new(exp_prefix),
                        "Prefix mismatch for '{}'",
                        input_str
                    );
                    assert_eq!(
                        res_suffix,
                        OsStr::new(exp_suffix),
                        "Suffix mismatch for '{}'",
                        input_str
                    );
                    println!(
                        "Input: \"{}\" -> Prefix: \"{}\", Suffix: \"{}\" (Correct)",
                        input_str,
                        res_prefix.to_string_lossy(),
                        res_suffix.to_string_lossy()
                    );
                }
                (None, None) => {
                    println!("Input: \"{}\" -> None (Correct)", input_str);
                }
                _ => {
                    panic!(
                        "Mismatch for input '{}': Expected {:?}, got {:?}",
                        input_str,
                        expected_output.map(|(p, s)| (
                            OsStr::new(p).to_string_lossy().into_owned(),
                            OsStr::new(s).to_string_lossy().into_owned()
                        )),
                        result.map(|(p, s)| (
                            p.to_string_lossy().into_owned(),
                            s.to_string_lossy().into_owned()
                        ))
                    );
                }
            }
        }
    }
}
