use std::{
    ffi::{OsStr, OsString},
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
#[cfg(windows)]
use std::os::windows::ffi::OsStringExt;

use image::ImageFormat;

use crate::{arg_parse_err::ArgParseErr, error::MagickError, wm_err};

use super::{Geometry, ResizeGeometry};

#[derive(Clone, Debug, Default, PartialEq)]
pub enum Location {
    Path(PathBuf),
    #[default]
    Stdio,
}

impl Location {
    pub fn from_arg(arg: &OsStr) -> Self {
        if matches!(arg.as_encoded_bytes(), b"" | b"-") {
            Self::Stdio
        } else {
            Self::Path(PathBuf::from(arg))
        }
    }

    pub fn to_filename(&self) -> OsString {
        match self {
            Location::Path(path) => path.clone().into_os_string(),
            Location::Stdio => OsString::from("-"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileFormat {
    Format(ImageFormat),
    /// Encoding operation is present but is a no-op. On the CLI this is "null:" passed as filename.
    DoNotEncode,
}

impl FileFormat {
    /// Creates a format from the explicit specifier that precedes the filename,
    /// e.g. `png:my-file` or `null:`
    pub fn from_prefix(prefix: &str) -> Option<Self> {
        let lowercase_prefix = prefix.to_ascii_lowercase();
        let format = if lowercase_prefix == "null" {
            Self::DoNotEncode
        } else {
            Self::Format(ImageFormat::from_extension(lowercase_prefix)?)
        };
        Some(format)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InputFileArg {
    pub location: Location,
    pub format: Option<FileFormat>,
    pub read_mod: Option<ReadModifier>,
}

impl InputFileArg {
    pub fn parse(input: &OsStr) -> Result<Self, MagickError> {
        Self::parse_inner(input, |path| {
            let metadata = std::fs::metadata(path);
            metadata.map(|d| d.is_dir())
        })
    }

    // `is_dir` checks whether a path is (a symlink to) a directory.
    // It's split out so that the parser can be tested without relying on external state.
    fn parse_inner(
        input: &OsStr,
        is_dir: impl Fn(&Path) -> Result<bool, std::io::Error>,
    ) -> Result<Self, MagickError> {
        // The basic syntax here is "format:path[modifier]", e.g. "jpg:foo.jpg[50x50]",
        // with the format and modifier being optional.
        // ImageMagick will try to look for the full input string first, so if a file named
        // "jpg:foo.jpg[50x50]" exists, it will be returned with empty format and modifier
        // (regardless of whether we have permission to read the file).
        // If "jpg:foo.jpg[50x50]" doesn't exist or is (a symlink to) a directory,
        // the next path tried is "jpg:foo.jpg" (with modifier=50x50),
        // and then the next one is "foo.jpg" (with format=jpg, modifier=50x50).
        // If none of those files exist:
        // - If "foo.jpg" exists as a directory, just return it anyway.
        //   The decoder will deal with the problem later.
        // - If "foo.jpg" doesn't exist, return error showing "foo.jpg" as the path,
        //   not the full original string.
        // If at any point in this process the remaining path becomes "" or "-",
        // we return the special Stdio location indicating that the input is from stdin.

        // A ready-to-return location is either stdin or an existing non-directory path.
        // It doesn't matter if it's a file that cannot be read, it should still be returned.
        let ready_to_return = |location: &Location| match location {
            Location::Path(path) => matches!(is_dir(path), Ok(false)),
            Location::Stdio => true,
        };

        // Try the input verbatim
        let location = Location::from_arg(input);
        if ready_to_return(&location) {
            return Ok(Self {
                location,
                format: None,
                read_mod: None,
            });
        }

        // Parse any read modifier, e.g. "foo.jpg[50x50]"
        let parse_result = split_off_bracketed_suffix(input).and_then(|(path, modifier)| {
            if let Ok(valid_modifier) = ReadModifier::try_from(modifier.as_ref()) {
                Some((path, valid_modifier))
            } else {
                // If something looks like a modifier but is not a valid modifier,
                // it is considered part of the path.
                // There is no "error: invalid modifier" error state.
                None
            }
        });
        let (path, read_mod) = match parse_result {
            Some((path, modifier)) => {
                let location = Location::from_arg(&path);
                if ready_to_return(&location) {
                    return Ok(Self {
                        location,
                        format: None,
                        read_mod: Some(modifier),
                    });
                }
                (path, Some(modifier))
            }
            // If a valid modifier is not found then the original string is used as is
            None => (input.to_owned(), None),
        };

        // Parse any explicitly-specified image format, e.g. "jpg:foo.jpg".
        // This must only be done after the read modifier ("[...]") is removed
        // to prevent splitting "[x:y]" (aspect ratio) modifier.
        let (path, format) = match parse_path_and_format(&path) {
            Some((path, format)) => {
                let location = Location::from_arg(&path);
                if ready_to_return(&location) {
                    return Ok(Self {
                        location,
                        format: Some(format),
                        read_mod,
                    });
                }
                (path, Some(format))
            }
            None => (path, None),
        };

        // At this point, everything we tried failed
        match is_dir(Path::new(&path)) {
            // This means the remaining path after modifier & format removal is a directory.
            // ImageMagick simply returns this path (as mentioned, without modifier & format).
            // Any error will come later from the decoder.
            Ok(true) => Ok(Self {
                location: Location::Path(PathBuf::from(path)),
                format,
                read_mod,
            }),
            // A normal file would have been returned earlier
            Ok(false) => unreachable!(),
            // `metadata` failed, maybe the file doesn't exist
            Err(error) => Err(wm_err!(
                "unable to open image '{}': {error}",
                input.to_string_lossy()
            )),
        }
    }
}

/// The action to be taken upon loading the image.
/// `convert` accepts any single one of: frame selection, resize, or crop.
///
/// See <https://imagemagick.org/Usage/files/#read_mods> for details.
/// I've also verified it behaves according to the documentation.
#[derive(Debug, Clone, PartialEq)]
pub enum ReadModifier {
    Resize(ResizeGeometry),
    Crop(LoadCropGeometry),
    // TODO: actually parse this nonsense.
    // So selectors can be `[5-7]` or negative like `[-5--7]`,
    // and that's all documented.
    // Did you know that `[+5-7]` is valid AND different from `[5-7]`?
    // I don't know why. I shouldn't have to wonder why.
    FrameSelect(OsString),
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
            Ok(Self::Resize(ResizeGeometry::try_from(s).map_err(
                |e| match &e.message {
                    Some(msg) => wm_err!("invalid resize geometry: {msg}"),
                    None => wm_err!("invalid resize geometry"),
                },
            )?))
        } else if x_count == 1 && plus_count == 2 {
            match LoadCropGeometry::try_from(s) {
                Ok(geom) => Ok(Self::Crop(geom)),
                Err(_) => Err(wm_err!("invalid crop geometry: {s:?}")),
            }
        } else {
            if ascii
                .iter()
                .all(|c| c.is_ascii_digit() || *c == b'-' || *c == b'+' || *c == b',')
            {
                Ok(Self::FrameSelect(s.to_owned()))
            } else {
                return Err(wm_err!("invalid read modifier: {}", s.to_string_lossy()));
            }
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
    type Err = ArgParseErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(OsStr::new(s))
    }
}

impl TryFrom<&OsStr> for LoadCropGeometry {
    type Error = ArgParseErr;

    fn try_from(s: &OsStr) -> Result<Self, Self::Error> {
        let geom = Geometry::try_from(s)?;

        let convert_field = |field: Option<f64>| -> Result<u32, ArgParseErr> {
            let f = field.ok_or(ArgParseErr::new())?;
            if f.is_sign_negative() {
                Err(ArgParseErr::new())
            } else {
                // imagemagick rounds to nearest, while plain `as` would round down
                Ok(f.round() as u32)
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

fn split_off_bracketed_suffix(input: &OsStr) -> Option<(OsString, OsString)> {
    // TODO: get rid of this platform-specific code once OsString.truncate() is stabilized:
    // https://doc.rust-lang.org/stable/std/ffi/struct.OsString.html#method.truncate
    // The bracketed suffix must be ascii, so we can split it off into a `&[u8]` and truncate the rest.
    #[cfg(any(unix, target_os = "wasi"))]
    {
        let bytes = input.as_bytes(); // Provided by std::os::unix::ffi::OsStrExt

        if bytes.is_empty() || bytes.last() != Some(&b']') {
            return None;
        }

        let slice_before_closing_bracket = &bytes[0..bytes.len() - 1];

        match slice_before_closing_bracket
            .iter()
            .rposition(|&b| b == b'[')
        {
            Some(open_bracket_idx) => {
                let prefix_bytes = &bytes[0..open_bracket_idx];
                let inner_content_bytes = &bytes[open_bracket_idx + 1..bytes.len() - 1];

                // OsStr::from_bytes is available via std::os::unix::ffi::OsStrExt
                // Convert to OsString for consistent return type with Windows path.
                let prefix_os_string = OsStr::from_bytes(prefix_bytes).to_os_string();
                let inner_content_os_string = OsStr::from_bytes(inner_content_bytes).to_os_string();
                Some((prefix_os_string, inner_content_os_string))
            }
            None => None,
        }
    }
    #[cfg(windows)]
    {
        // Use encode_wide and from_wide on Windows to avoid `unsafe` I'm not confident in.
        // OsStr::encode_wide() is provided by std::os::windows::ffi::OsStrExt
        let wide_chars: Vec<u16> = input.encode_wide().collect();

        if wide_chars.is_empty() {
            return None;
        }
        // Check the last wide character for ']'
        // ']' as a u16 is 93.
        if wide_chars.last() != Some(&(b']' as u16)) {
            return None;
        }

        // Search for the last '[' before the final ']'.
        // Slice of wide_chars excluding the last ']'
        let slice_before_closing_bracket = &wide_chars[0..wide_chars.len() - 1];

        // '[' as a u16 is 91.
        match slice_before_closing_bracket
            .iter()
            .rposition(|&wc| wc == (b'[' as u16))
        {
            Some(open_bracket_u16_idx) => {
                let prefix_u16_slice = &wide_chars[0..open_bracket_u16_idx];
                let inner_u16_slice = &wide_chars[open_bracket_u16_idx + 1..wide_chars.len() - 1];

                // OsString::from_wide is provided by std::os::windows::ffi::OsStringExt
                let prefix_os_string = OsString::from_wide(prefix_u16_slice);
                let inner_os_string = OsString::from_wide(inner_u16_slice);
                Some((prefix_os_string, inner_os_string))
            }
            None => {
                // Found ']' at the end, but no matching '[' before it.
                None
            }
        }
    }
    #[cfg(not(any(unix, windows, target_os = "wasi")))]
    {
        // Fallback for other platforms:
        // This is a simplified attempt. Real-world handling for other platforms
        // would depend on their OsStr specifics and available APIs in older Rust.
        // The most portable thing to try is to_str() if the content is UTF-8.
        if let Some(s_ref) = input.to_str() {
            let bytes = s_ref.as_bytes();
            if bytes.is_empty() || bytes.last() != Some(&b']') {
                return None;
            }
            let slice_before_closing_bracket = &bytes[0..bytes.len() - 1];
            match slice_before_closing_bracket
                .iter()
                .rposition(|&b| b == b'[')
            {
                Some(open_bracket_idx) => {
                    let prefix_str_slice = &s_ref[0..open_bracket_idx];
                    let inner_content_str_slice = &s_ref[open_bracket_idx + 1..bytes.len() - 1];
                    Some((
                        OsString::from(prefix_str_slice), // Convert &str to OsString
                        OsString::from(inner_content_str_slice),
                    ))
                }
                None => None,
            }
        } else {
            // non-utf-8 path on an unknown platform
            None
        }
    }
}

/// Parses ImageMagick `format:path`-style argument.
pub fn parse_path_and_format(input: &OsStr) -> Option<(OsString, FileFormat)> {
    #[cfg(any(unix, target_os = "wasi"))]
    {
        let bytes = input.as_bytes(); // From std::os::unix::ffi::OsStrExt
        let mut iter = bytes.splitn(2, |&b| b == b':');
        let prefix = str::from_utf8(iter.next().unwrap()).ok()?;
        let suffix = iter.next()?;
        Some((
            OsStr::from_bytes(suffix).to_owned(), // From std::os::unix::ffi::OsStrExt
            FileFormat::from_prefix(prefix)?,
        ))
    }
    #[cfg(windows)]
    {
        let wide_chars: Vec<u16> = input.encode_wide().collect(); // From std::os::windows::ffi::OsStrExt
        let mut iter = wide_chars.splitn(2, |&wc| wc == b':' as u16);
        let prefix = String::from_utf16(iter.next().unwrap()).ok()?;
        // On Windows, ImageMagick treats "c:..." as a path
        if prefix.len() == 1 && prefix.chars().nth(0).map(|c| c.is_ascii_alphabetic()) == Some(true)
        {
            return None;
        }
        let suffix = iter.next()?;
        Some((
            OsString::from_wide(suffix), // From std::os::windows::ffi::OsStringExt
            FileFormat::from_prefix(&prefix)?,
        ))
    }
    #[cfg(not(any(unix, windows, target_os = "wasi")))]
    {
        // Outside the above platforms, we only support splitting UTF-8
        let input = input.to_str()?;
        let (prefix, suffix) = input.split_once(':')?;
        Some((OsString::from(suffix), FileFormat::from_prefix(prefix)?))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use image::ImageFormat;

    use super::*;

    /// Creates a mock is_dir closure for the parse tests.
    /// The resulting closure pretends that the specified files and dirs exist.
    fn mock_is_dir(
        files: impl Into<Vec<&'static str>>,
        dirs: impl Into<Vec<&'static str>>,
    ) -> impl Fn(&Path) -> Result<bool, std::io::Error> {
        let files = files.into();
        let dirs = dirs.into();
        move |path| {
            if files.iter().any(|f| Path::new(f) == path) {
                Ok(false)
            } else if dirs.iter().any(|d| Path::new(d) == path) {
                Ok(true)
            } else {
                Err(std::io::ErrorKind::NotFound.into())
            }
        }
    }

    #[test]
    fn parse_path() {
        // Simple parsing test; see the modifier & format tests for more
        assert_eq!(
            InputFileArg::parse_inner(
                OsStr::new("jpg:file.png[1x2+3+4]"),
                mock_is_dir(["file.png"], [])
            )
            .unwrap(),
            InputFileArg {
                location: Location::Path("file.png".into()),
                format: Some(FileFormat::Format(ImageFormat::Jpeg)),
                read_mod: Some(ReadModifier::Crop(
                    LoadCropGeometry::from_str("1x2+3+4").unwrap()
                ))
            }
        );

        // Tricky interactions with existing files/directories
        let result = InputFileArg::parse_inner(
            OsStr::new("file.png[1x2+3+4]"),
            mock_is_dir(["file.png[1x2+3+4]"], []),
        );
        assert_eq!(
            result.unwrap().location,
            Location::Path("file.png[1x2+3+4]".into()),
        );
        let result = InputFileArg::parse_inner(
            OsStr::new("file.png[1x2+3+4]"),
            mock_is_dir(["file.png"], ["file.png[1x2+3+4]"]),
        );
        assert_eq!(result.unwrap().location, Location::Path("file.png".into()));
        let result = InputFileArg::parse_inner(
            OsStr::new("file.png[1x2+3+4]"),
            mock_is_dir([], ["file.png[1x2+3+4]"]),
        );
        assert!(result.is_err());
    }

    #[test]
    fn parse_stdin() {
        // Simple cases (no weirdly-named files in the current dir)
        for path in [
            "",
            "-",
            "png:",
            "png:-",
            "[1x1]",
            "-[1x1]",
            "png:[1x1]",
            "png:-[1x1]",
        ] {
            let result = InputFileArg::parse_inner(OsStr::new(path), mock_is_dir([], []));
            assert_eq!(result.unwrap().location, Location::Stdio, "{path:?}");
        }

        // What if there is a file named "-"?
        let result = InputFileArg::parse_inner(OsStr::new("-"), mock_is_dir(["-"], []));
        assert_eq!(result.unwrap().location, Location::Stdio);
        let result = InputFileArg::parse_inner(OsStr::new("png:-[1x1]"), mock_is_dir(["-"], []));
        assert_eq!(result.unwrap().location, Location::Stdio);

        // Unlike plain "-", the presence of these files prevent the path from being parsed as stdin
        let result = InputFileArg::parse_inner(OsStr::new("png:-"), mock_is_dir(["png:-"], []));
        assert_eq!(result.unwrap().location, Location::Path("png:-".into()));
        let result =
            InputFileArg::parse_inner(OsStr::new("png:-[1x1]"), mock_is_dir(["png:-[1x1]"], []));
        assert_eq!(
            result.unwrap().location,
            Location::Path("png:-[1x1]".into()),
        );
        let result =
            InputFileArg::parse_inner(OsStr::new("png:-[1x1]"), mock_is_dir(["png:-"], []));
        assert_eq!(result.unwrap().location, Location::Path("png:-".into()));
    }

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

            match (&result, expected_output) {
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

    #[test]
    fn test_parse_path_and_format() {
        assert_eq!(parse_path_and_format(OsStr::new("file.png")), None);
        assert_eq!(
            parse_path_and_format(OsStr::new("jpg:file.png")),
            Some((
                OsString::from("file.png"),
                FileFormat::Format(ImageFormat::Jpeg)
            )),
        );
        assert_eq!(
            parse_path_and_format(OsStr::new("jpeg:file.png")),
            Some((
                OsString::from("file.png"),
                FileFormat::Format(ImageFormat::Jpeg)
            )),
        );
        assert_eq!(
            parse_path_and_format(OsStr::new("JPEG:file.png")),
            Some((
                OsString::from("file.png"),
                FileFormat::Format(ImageFormat::Jpeg)
            )),
        );
        assert_eq!(
            parse_path_and_format(OsStr::new("jpg:")),
            Some((OsString::from(""), FileFormat::Format(ImageFormat::Jpeg))),
        );
        assert_eq!(parse_path_and_format(OsStr::new("")), None);
        assert_eq!(parse_path_and_format(OsStr::new(":")), None);
        assert_eq!(parse_path_and_format(OsStr::new(":file.png")), None);
        assert_eq!(
            parse_path_and_format(OsStr::new("unsupported_format:file.png")),
            None,
        );
    }

    #[test]
    fn test_parse_null_format() {
        assert_eq!(
            parse_path_and_format(OsStr::new("null:file.png")),
            Some((OsString::from("file.png"), FileFormat::DoNotEncode)),
        );
        assert_eq!(
            parse_path_and_format(OsStr::new("null:")),
            Some((OsString::from(""), FileFormat::DoNotEncode)),
        );
    }
}
