use std::ffi::OsString;

use crate::args::Operation;

/// Plan of operations for the whole run over multiple files
#[derive(Debug, Default)]
pub struct ExecutionPlan {
    pub output_file: OsString,
    pub input_files: Vec<FilePlan>,
}

impl ExecutionPlan {
    pub fn add_operation(&mut self, op: Operation) {
        // Operations such as -resize apply to all the files already listed,
        // but not subsequent ones
        for file_plan in &mut self.input_files {
            file_plan.ops.push(op)
        }
    }
}

/// Plan of operations for a single input file
#[derive(Debug, Default)]
pub struct FilePlan {
    filename: OsString,
    ops: Vec<Operation>,
}
