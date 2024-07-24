//! Imagemagick argument parsing.
//!
//! We cannot use an argument parsing library because imagemagick arguments are unconventional:
//! they are prefixed by -, not --. So we need to hand-roll our own parser.

use std::{
    ffi::{OsStr, OsString},
    fmt::Display,
    str::FromStr,
};

use image::DynamicImage;

use crate::{
    arg_parsers::ResizeGeometry,
    error::MagickError,
    operations,
    plan::{ExecutionPlan, FilePlan},
    wm_err, wm_try,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Operation {
    Resize(ResizeGeometry),
    Thumbnail(ResizeGeometry),
}

impl Operation {
    // TODO: bubble up errors
    pub fn execute(&self, image: &mut DynamicImage) {
        match self {
            Operation::Resize(geom) => operations::resize::resize(image, geom),
            Operation::Thumbnail(geom) => operations::resize::thumbnail(image, geom),
        }
    }
}

#[derive(Debug)]
pub struct ArgParseErr {}

impl Display for ArgParseErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to parse arguments") // TODO: elaborate
    }
}

impl std::error::Error for ArgParseErr {}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Arg {
    Resize,
    Thumbnail,
}

impl FromStr for Arg {
    type Err = MagickError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "-resize" => Ok(Arg::Resize),
            "-thumbnail" => Ok(Arg::Thumbnail),
            _ => Err(wm_err!(format!("unrecognized option `{}'", s))),
        }
    }
}

impl Arg {
    fn needs_value(&self) -> bool {
        match self {
            Arg::Resize => true,
            Arg::Thumbnail => true,
        }
    }

    fn to_operation(&self, value: Option<&OsStr>) -> Result<Operation, MagickError> {
        if self.needs_value() != value.is_some() {
            return Err(wm_err!(format!("argument requires a value")));
        };

        match self {
            Arg::Resize => Ok(Operation::Resize(wm_try!(ResizeGeometry::try_from(
                value.unwrap()
            )))),
            Arg::Thumbnail => Ok(Operation::Thumbnail(wm_try!(ResizeGeometry::try_from(
                value.unwrap()
            )))),
        }
    }
}

pub fn maybe_print_help() {
    match std::env::args_os().nth(1) {
        None => print_help_and_exit(),
        Some(arg) => {
            if arg.as_os_str() == OsStr::new("--help") || arg.as_os_str() == OsStr::new("-help") {
                print_help_and_exit()
            }
        }
    }
}

fn print_help_and_exit() -> ! {
    todo!();
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
    if starts_with_dash(&output_filename) {
        return Err(wm_err!(format!(
            "missing an image filename `{}'",
            output_filename.to_string_lossy()
        )));
    }

    let mut plan = ExecutionPlan::default();
    plan.output_file = output_filename;

    // TODO: parse the filename specification, there's a lot of operations that can be attached to it

    let mut iter = args.into_iter().skip(1); // skip argv[0], path to our binary
    while let Some(arg) = iter.next() {
        if arg.as_encoded_bytes() == [b'-'] {
            todo!(); // this is stdin or stdout
        } else if starts_with_dash(&arg) {
            // A file named "-foobar.jpg" will be parsed as an option.
            // Sadly imagemagick does not support the -- convention to separate options and filenames,
            // and there is nothing we can do about it without introducing incompatibility in argument parsing.
            let string_arg = arg.to_str().ok_or(wm_err!(format!(
                "unrecognized option `{}'",
                arg.to_string_lossy()
            )))?;
            let arg_name = Arg::from_str(string_arg)?;
            let operation = if arg_name.needs_value() {
                let value = iter.next().ok_or(wm_err!(format!(
                    "argument requires a value: {}",
                    &string_arg
                )))?;
                arg_name.to_operation(Some(value.as_os_str()))?
            } else {
                arg_name.to_operation(None)?
            };
            plan.add_operation(operation);
        } else {
            plan.input_files.push(FilePlan::new(arg));
        }
    }
    Ok(plan)
}

fn starts_with_dash(arg: &OsStr) -> bool {
    arg.as_encoded_bytes().get(0) == Some(&b'-')
    // Anything starting with two dashes instead of one is treated as filename
    && arg.as_encoded_bytes().get(1) != Some(&b'-')
}
