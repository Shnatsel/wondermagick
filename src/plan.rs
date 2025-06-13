use std::ffi::{OsStr, OsString};

use crate::arg_parsers::InputFileArg;
use crate::decode::decode;
use crate::filename_utils::insert_suffix_before_extension_in_path;
use crate::{error::MagickError, operations::Operation, wm_try};

/// Plan of operations for the whole run over multiple files
#[derive(Debug, Default)]
pub struct ExecutionPlan {
    /// Operations to be applied to ALL input files
    global_ops: Vec<Operation>,
    pub output_file: OsString,
    pub input_files: Vec<FilePlan>,
}

impl ExecutionPlan {
    pub fn add_operation(&mut self, op: Operation) {
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
        for (file_plan, output_file) in self.input_files.iter().zip(self.output_filenames().iter())
        {
            let mut image = wm_try!(decode(&file_plan.filename, None));

            for operation in &file_plan.ops {
                operation.execute(&mut image)?;
            }

            wm_try!(image.save(output_file));
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
