//! Imagemagick argument parsing.
//!
//! We cannot use an argument parsing library because imagemagick arguments are unconventional:
//! they are prefixed by -, not --. So we need to hand-roll our own parser.

use std::ffi::{OsStr, OsString};

use crate::{arg_parsers::InputFileArg, error::MagickError, plan::ExecutionPlan, wm_err};

use strum::{EnumString, IntoStaticStr, VariantArray};

#[derive(EnumString, IntoStaticStr, VariantArray, Debug, Clone, Copy, PartialEq, Eq)]
#[strum(serialize_all = "kebab-case")]
pub enum Arg {
    Crop,
    Resize,
    Thumbnail,
    Scale,
    Sample,
    AutoOrient,
    Quality,
}

impl Arg {
    pub fn needs_value(&self) -> bool {
        match self {
            Arg::Crop => true,
            Arg::Resize => true,
            Arg::Thumbnail => true,
            Arg::Scale => true,
            Arg::Sample => true,
            Arg::AutoOrient => false,
            Arg::Quality => true,
        }
    }

    pub fn help_text(&self) -> &'static str {
        match self {
            Arg::Crop => "cut out a rectangular region of the image",
            Arg::Resize => "resize the image",
            Arg::Thumbnail => "create a thumbnail of the image",
            Arg::Scale => "scale the image",
            Arg::Sample => "scale image with pixel sampling",
            Arg::AutoOrient => "automagically orient (rotate) image",
            Arg::Quality => "JPEG/MIFF/PNG compression level", // I'm so sorry
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
    if starts_with_sign(&output_filename) {
        return Err(wm_err!(
            "missing an image filename `{}'",
            output_filename.to_string_lossy()
        ));
    }

    let mut plan = ExecutionPlan::default();
    plan.set_output_file(output_filename);

    let mut iter = args.into_iter().skip(1); // skip argv[0], path to our binary
    while let Some(raw_arg) = iter.next() {
        if raw_arg.as_encoded_bytes() == [b'-'] {
            todo!(); // this is stdin or stdout
        } else if starts_with_sign(&raw_arg) {
            // A file named "-foobar.jpg" will be parsed as an option.
            // Sadly imagemagick does not support the -- convention to separate options and filenames,
            // and there is nothing we can do about it without introducing incompatibility in argument parsing.
            let (_sign, string_arg) = sign_and_arg_name(raw_arg)?;
            let arg = Arg::try_from(string_arg.as_str())
                .map_err(|_| wm_err!("unrecognized option `{}'", string_arg))?;
            let value = if arg.needs_value() { iter.next() } else { None };
            plan.apply_arg(arg, value.as_deref())?;
        } else {
            plan.add_input_file(InputFileArg::parse(&raw_arg)?);
        }
    }
    Ok(plan)
}

/// Checks if the string starts with a `-` or a `+`
fn starts_with_sign(arg: &OsStr) -> bool {
    let first_byte = arg.as_encoded_bytes().first();
    first_byte == Some(&b'-')
        || first_byte == Some(&b'+')
    // Anything starting with two dashes instead of one is treated as filename
    && arg.as_encoded_bytes().get(1) != Some(&b'-')
}

/// Splits the string into a sign (- or +) and argument name
fn sign_and_arg_name(raw_arg: OsString) -> Result<(u8, String), MagickError> {
    let mut string = raw_arg
        .into_string()
        .map_err(|s| wm_err!("unrecognized option `{}'", s.to_string_lossy()))?;
    let sign = string.remove(0);
    assert!(sign == '-' || sign == '+');
    Ok((sign as u8, string))
}
