//! Imagemagick argument parsing.
//!
//! We cannot use an argument parsing library because imagemagick arguments are unconventional:
//! they are prefixed by -, not --. So we need to hand-roll our own parser.

use std::{
    ffi::{OsStr, OsString},
    path::Path,
};

use crate::{
    arg_parsers::{parse_path_and_format, FileFormat, InputFileArg, Location},
    error::MagickError,
    plan::ExecutionPlan,
    wm_err,
};

use strum::{EnumString, IntoStaticStr, VariantArray};

pub enum ArgSign {
    Plus,
    Minus,
}

impl TryFrom<char> for ArgSign {
    type Error = MagickError;

    fn try_from(value: char) -> Result<Self, MagickError> {
        match value {
            '-' => Ok(ArgSign::Minus),
            '+' => Ok(ArgSign::Plus),
            _ => Err(wm_err!("invalid argument sign `{}'", value)),
        }
    }
}

/// Some arguments have different behavior depending on whether they are prefixed with `-` or `+`.
pub struct SignedArg {
    pub sign: ArgSign,
    pub arg: Arg,
}

impl SignedArg {
    pub fn needs_value(&self) -> bool {
        self.arg.needs_value()
    }
}

#[derive(EnumString, IntoStaticStr, VariantArray, Debug, Clone, Copy, PartialEq, Eq)]
#[strum(serialize_all = "kebab-case")]
pub enum Arg {
    AutoOrient,
    Crop,
    // TODO: -format can actually change meaning, as `-format type`
    // and as `-format expression`. We currently only implement `-format expression`.
    Format,
    Filter,
    Flip,
    Flop,
    Blur,
    GaussianBlur,
    Identify,
    Monochrome,
    Negate,
    Quality,
    Resize,
    Sample,
    Scale,
    Strip,
    Thumbnail,
}

impl Arg {
    pub fn needs_value(&self) -> bool {
        match self {
            Arg::AutoOrient => false,
            Arg::Crop => true,
            Arg::Format => true,
            Arg::Filter => true,
            Arg::Flip => false,
            Arg::Flop => false,
            Arg::Blur => true,
            Arg::GaussianBlur => true,
            Arg::Identify => false,
            Arg::Monochrome => false,
            Arg::Negate => false,
            Arg::Quality => true,
            Arg::Resize => true,
            Arg::Sample => true,
            Arg::Scale => true,
            Arg::Strip => false,
            Arg::Thumbnail => true,
        }
    }

    pub fn help_text(&self) -> &'static str {
        match self {
            Arg::AutoOrient => "automagically orient (rotate) image",
            Arg::Crop => "cut out a rectangular region of the image",
            Arg::Format => "output formatted image characteristics",
            Arg::Filter => "use this filter when resizing an image",
            Arg::Flip => "flip image vertically",
            Arg::Flop => "flop image horizontally",
            Arg::Blur => "reduce image noise and reduce detail levels",
            Arg::GaussianBlur => "reduce image noise and reduce detail levels",
            Arg::Identify => "identify the format and characteristics of the image",
            Arg::Monochrome => "transform image to black and white",
            Arg::Negate => "replace every pixel with its complementary color",
            Arg::Quality => "JPEG/MIFF/PNG compression level", // I'm so sorry
            Arg::Resize => "resize the image",
            Arg::Sample => "scale image with pixel sampling",
            Arg::Scale => "scale the image",
            Arg::Strip => "strip image of all profiles and comments",
            Arg::Thumbnail => "create a thumbnail of the image",
        }
    }
}

pub fn parse_args(mut args: Vec<OsString>) -> Result<ExecutionPlan, MagickError> {
    // TODO: whole lotta stuff: https://imagemagick.org/script/command-line-processing.php

    // maybe_print_help should take care of it, but this won't hurt
    if args.len() <= 1 {
        return Err(wm_err!("No command-line arguments provided"));
    }

    // imagemagick seems to first determine the output filename, and complains if it's not right.
    // Contrary to the documentation about -flags being treated as filenames by default,
    // the observed behavior on my system is that they're only ever parsed as flags.
    let output_filename = args.pop().unwrap();
    // imagemagick rejects output filenames that look like arguments
    if optionlike(&output_filename) {
        return Err(wm_err!(
            "missing output filename `{}'",
            output_filename.to_string_lossy()
        ));
    }

    let mut plan = ExecutionPlan::default();
    let (output_file, output_format) = parse_output_file(&output_filename, |path| {
        matches!(std::fs::exists(path), Ok(true))
    });
    plan.set_output_file(output_file, output_format);

    let mut iter = args.into_iter().skip(1); // skip argv[0], path to our binary
    while let Some(raw_arg) = iter.next() {
        if optionlike(&raw_arg) {
            // A file named "-foobar.jpg" will be parsed as an option.
            // Sadly imagemagick does not support the -- convention to separate options and filenames,
            // and there is nothing we can do about it without introducing incompatibility in argument parsing.
            let (sign, string_arg) = sign_and_arg_name(raw_arg)?;
            let arg = Arg::try_from(string_arg.as_str())
                .map_err(|_| wm_err!("unrecognized option `{}'", string_arg))?;
            let value = if arg.needs_value() { iter.next() } else { None };
            plan.apply_arg(SignedArg { sign, arg }, value.as_deref())?;
        } else {
            plan.add_input_file(InputFileArg::parse(&raw_arg)?);
        }
    }
    Ok(plan)
}

fn parse_output_file(
    input: &OsStr,
    exists: impl FnOnce(&Path) -> bool,
) -> (Location, Option<FileFormat>) {
    let mut output_file = Location::from_arg(input);
    let mut output_format = None;
    // "-" is parsed as (Stdio, None) no matter what.
    // "png:-" is parsed as:
    //   - (Path("png:-"), None) if a file or dir named "png:-" exists.
    //   - (Stdio, Some("png")) otherwise.
    if let Location::Path(path) = &output_file {
        if !exists(path) {
            if let Some((path, format)) = parse_path_and_format(input) {
                output_file = Location::from_arg(&path);
                output_format = Some(format);
            }
        }
    }
    (output_file, output_format)
}

/// Checks if the string starts with a `-` or a `+`, followed by an ASCII letter
fn optionlike(arg: &OsStr) -> bool {
    matches!(
        arg.as_encoded_bytes(),
        [b'-' | b'+', b'a'..=b'z' | b'A'..=b'Z', ..],
    )
}

/// Splits the string into a sign (- or +) and argument name
fn sign_and_arg_name(raw_arg: OsString) -> Result<(ArgSign, String), MagickError> {
    let mut string = raw_arg
        .into_string()
        .map_err(|s| wm_err!("unrecognized option `{}'", s.to_string_lossy()))?;
    let sign = string.remove(0);
    Ok((ArgSign::try_from(sign)?, string))
}

#[cfg(test)]
mod tests {
    use image::ImageFormat;
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_parse_output_file() {
        assert_eq!(
            parse_output_file(OsStr::new("-"), |_| false),
            (Location::Stdio, None),
        );
        assert_eq!(
            parse_output_file(OsStr::new("-"), |_| true),
            (Location::Stdio, None),
        );
        assert_eq!(
            parse_output_file(OsStr::new("png:-"), |_| false),
            (Location::Stdio, Some(FileFormat::Format(ImageFormat::Png))),
        );
        assert_eq!(
            parse_output_file(OsStr::new("png:-"), |_| true),
            (Location::Path(PathBuf::from("png:-")), None),
        );
    }
}
