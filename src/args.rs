//! Imagemagick argument parsing.
//!
//! We cannot use an argument parsing library because imagemagick arguments are unconventional:
//! they are prefixed by -, not --. So we need to hand-roll our own parser.

use std::{
    ffi::{OsStr, OsString},
    fmt::Display,
    str::FromStr,
};

pub struct Plan {
    pub output_file: OsString,
    pub input_files: Vec<FilePlan>,
}

impl Plan {
    pub fn process_arg() {
        todo!()
    }
}

pub struct FilePlan {
    filename: OsString,
    ops: Vec<Operation>,
}

pub enum Operation {
    Resize,
}

#[derive(Debug)]
pub struct ArgParseErr {}

impl Display for ArgParseErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to parse arguments") // TODO: elaborate
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Arg {
    Resize,
}

impl FromStr for Arg {
    type Err = ArgParseErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "-resize" => Ok(Arg::Resize),
            _ => Err(ArgParseErr {}),
        }
    }
}

impl Arg {
    fn needs_value(&self) -> bool {
        match self {
            Arg::Resize => true,
        }
    }

    fn to_operation(&self, value: Option<&OsStr>) -> Result<Operation, ArgParseErr> {
        if self.needs_value() != value.is_some() {
            return Err(ArgParseErr {});
        };

        if !self.needs_value() {
            // TODO: flags go here
        } else {
        }

        todo!()
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

pub fn parse_args(mut args: Vec<OsString>) -> Result<Plan, ArgParseErr> {
    // TODO: whole lotta stuff: https://imagemagick.org/script/command-line-processing.php

    // maybe_print_help should take care of it, but this won't hurt
    if args.len() <= 1 {
        return Err(ArgParseErr {});
    }

    // imagemagick seems to first determine the output filename, and complains if it's not right.
    // Contrary to the documentation about -flags being treated as filenames by default,
    // the observed behavior on my system is that they're only ever parsed as flags.
    let output_filename = args.pop().unwrap();
    // imagemagick rejects output filenames that look like arguments
    if starts_with_dash(&output_filename) {
        return Err(ArgParseErr {});
    }
    // TODO: parse the filename specification, there's a lot of operations that can be attached to it
    let mut input_filenames: Vec<OsString> = Vec::new();

    let mut iter = args.into_iter().skip(1); // skip argv[0], path to our binary
    while let Some(arg) = iter.next() {
        if arg.as_encoded_bytes() == [b'-'] {
            todo!(); // this is stdin or stdout
        } else if starts_with_dash(&arg) {
            // A file named "-foobar.jpg" will be parsed as an option.
            // Sadly imagemagick does not support the -- convention to separate options and filenames,
            // and there is nothing we can do about it without introducing incompatibility in argument parsing.
            let string_arg = arg.to_str().ok_or(ArgParseErr {})?;
            let arg_name = Arg::from_str(string_arg)?;
            if arg_name.needs_value() {
                let value = iter.next().ok_or(ArgParseErr {})?;
                // TODO: call the parser for this particular value
            }
        } else {
            input_filenames.push(arg);
        }
    }
    todo!(); // convert stuff into an execution plan
}

fn starts_with_dash(arg: &OsStr) -> bool {
    arg.as_encoded_bytes().get(0) == Some(&b'-')
    // Anything starting with two dashes instead of one is treated as filename
    && arg.as_encoded_bytes().get(1) != Some(&b'-')
}
