use std::ffi::OsString;

use crate::operations::Operation;

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

    pub fn add_input_file(&mut self, filename: OsString) {
        self.input_files.push(FilePlan {
            filename,
            ops: self.global_ops.clone(),
        });
    }
}

/// Plan of operations for a single input file
#[derive(Debug, Default)]
pub struct FilePlan {
    pub filename: OsString,
    pub ops: Vec<Operation>,
}
