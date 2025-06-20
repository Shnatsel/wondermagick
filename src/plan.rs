use std::ffi::{OsStr, OsString};

use crate::arg_parsers::{InputFileArg, ResizeGeometry};
use crate::args::Arg;
use crate::decode::decode;
use crate::filename_utils::insert_suffix_before_extension_in_path;
use crate::{encode, wm_err};
use crate::{error::MagickError, operations::Operation, wm_try};

/// Plan of operations for the whole run over multiple files
#[derive(Debug, Default)]
pub struct ExecutionPlan {
    /// Operations to be applied to ALL input files
    global_ops: Vec<Operation>,
    pub output_file: OsString,
    input_files: Vec<FilePlan>,
}

impl ExecutionPlan {
    pub fn apply_arg(&mut self, arg: Arg, value: Option<&OsStr>) -> Result<(), MagickError> {
        let arg_string: &'static str = arg.into();
        if arg.needs_value() != value.is_some() {
            return Err(wm_err!("argument requires a value: {arg_string}"));
        };

        self.apply_arg_inner(arg, value).map_err(|_err| {
            wm_err!(
                "invalid argument for option `-{arg_string}': {}",
                value.unwrap().to_string_lossy()
            )
        })?;

        Ok(())
    }

    /// Currently this can only fail due to argument parsing.
    /// Split into its own function due to lack of try{} blocks on stable Rust.
    /// TODO: make it return ArgParseErr and match imagemagick error reporting more closely
    fn apply_arg_inner(&mut self, arg: Arg, value: Option<&OsStr>) -> Result<(), MagickError> {
        match arg {
            Arg::Resize => {
                self.add_operation(Operation::Resize(ResizeGeometry::try_from(value.unwrap())?))
            }
            Arg::Thumbnail => self.add_operation(Operation::Thumbnail(ResizeGeometry::try_from(
                value.unwrap(),
            )?)),
            Arg::Scale => {
                self.add_operation(Operation::Scale(ResizeGeometry::try_from(value.unwrap())?))
            }
            Arg::Sample => {
                self.add_operation(Operation::Sample(ResizeGeometry::try_from(value.unwrap())?))
            }
            Arg::AutoOrient => self.add_operation(Operation::AutoOrient),
            Arg::Quality => todo!(),
        };

        Ok(())
    }

    fn add_operation(&mut self, op: Operation) {
        // Operations such as -resize apply to all the files already listed,
        // but not subsequent ones,
        // UNLESS they are specified before any of the files,
        // in which case they apply to all subsequent operations.
        if self.input_files.is_empty() {
            self.global_ops.push(op);
        } else {
            for file_plan in &mut self.input_files {
                file_plan.ops.push(op)
            }
        }
    }

    pub fn add_input_file(&mut self, file: InputFileArg) {
        let filename = file.path.into_os_string();

        let mut file_plan = FilePlan {
            filename,
            ops: self.global_ops.clone(),
        };

        if let Some(modifier) = file.read_mod {
            use crate::arg_parsers::ReadModifier::*;
            let op = match modifier {
                Resize(geom) => Some(Operation::Resize(geom)),
                Crop(geom) => Some(Operation::CropOnLoad(geom)),
                FrameSelect(s) => {
                    if s != OsStr::new("0") {
                        panic!("frame selection is not yet supported");
                    }
                    None
                }
            };
            if let Some(op) = op {
                file_plan.ops.insert(0, op);
            }
        }

        self.input_files.push(file_plan);
    }

    pub fn execute(&self) -> Result<(), MagickError> {
        if self.input_files.is_empty() {
            return Err(wm_err!("no images defined")); // mimics imagemagick
        }
        for (file_plan, output_file) in self.input_files.iter().zip(self.output_filenames().iter())
        {
            let mut image = wm_try!(decode(&file_plan.filename, None));

            for operation in &file_plan.ops {
                operation.execute(&mut image)?;
            }

            encode::encode(&image, &output_file, None, None)?;
        }

        Ok(())
    }

    fn output_filenames(&self) -> Vec<OsString> {
        if self.input_files.len() == 1 {
            vec![self.output_file.clone()]
        } else {
            let mut names = Vec::new();
            for i in 1..=self.input_files.len() {
                let name = self.output_file.clone();
                let suffix = OsString::from(format!("-{}", i)); // indexing for output images starts at 1
                let name = insert_suffix_before_extension_in_path(&name, &suffix);
                names.push(name);
            }
            names
        }
    }
}

/// Plan of operations for a single input file
#[derive(Debug, Default)]
pub struct FilePlan {
    pub filename: OsString,
    pub ops: Vec<Operation>,
}
