use std::{
    ffi::{OsStr, OsString},
    path::PathBuf,
};

use image::ImageFormat;

use crate::arg_parse_err::ArgParseErr;
use crate::arg_parsers::{
    parse_numeric_arg, CropGeometry, IdentifyFormat, InputFileArg, Location, ResizeGeometry,
};
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
    output_file: Location,
    input_files: Vec<FilePlan>,
    output_format: Option<ImageFormat>,
    modifiers: Modifiers,
}

impl ExecutionPlan {
    pub fn apply_arg(&mut self, arg: Arg, value: Option<&OsStr>) -> Result<(), MagickError> {
        let arg_string: &'static str = arg.into();
        if arg.needs_value() != value.is_some() {
            return Err(wm_err!("argument requires a value: {arg_string}"));
        };

        self.apply_arg_inner(arg, value).map_err(|arg_err| {
            wm_err!(
                "{}",
                arg_err.display_with_arg(arg_string, value.unwrap_or_default())
            )
        })?;

        Ok(())
    }

    /// Currently this can only fail due to argument parsing.
    /// Split into its own function due to lack of try{} blocks on stable Rust.
    fn apply_arg_inner(&mut self, arg: Arg, value: Option<&OsStr>) -> Result<(), ArgParseErr> {
        match arg {
            Arg::AutoOrient => self.add_operation(Operation::AutoOrient),
            Arg::Crop => {
                self.add_operation(Operation::Crop(CropGeometry::try_from(value.unwrap())?))
            }
            Arg::Identify => {
                self.add_operation(Operation::Identify(self.modifiers.identify_format.clone()));
            }
            Arg::Quality => self.modifiers.quality = Some(parse_numeric_arg(value.unwrap())?),
            Arg::Resize => {
                self.add_operation(Operation::Resize(ResizeGeometry::try_from(value.unwrap())?))
            }
            Arg::Sample => {
                self.add_operation(Operation::Sample(ResizeGeometry::try_from(value.unwrap())?))
            }
            Arg::Scale => {
                self.add_operation(Operation::Scale(ResizeGeometry::try_from(value.unwrap())?))
            }
            Arg::Strip => {
                self.modifiers.strip.set_all(true);
            }
            Arg::Thumbnail => {
                self.add_operation(Operation::Thumbnail(ResizeGeometry::try_from(
                    value.unwrap(),
                )?));
                // -thumbnail also strips all metadata except the ICC profile
                // Some docs state that it strips ICC profile also, but
                // https://usage.imagemagick.org/thumbnails/ says v6.5.4-7 onwards preserves them.
                self.modifiers.strip.set_all(true);
                self.modifiers.strip.icc = false;
            }
            Arg::Format => {
                self.modifiers.identify_format = Some(IdentifyFormat::try_from(value.unwrap())?)
            }
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
                file_plan.ops.push(op.clone())
            }
        }
    }

    pub fn add_input_file(&mut self, file: InputFileArg) {
        let mut file_plan = FilePlan {
            location: file.location,
            format: file.format,
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

    pub fn set_output_file(&mut self, file: Location, format: Option<ImageFormat>) {
        self.output_file = file;
        self.output_format = format;
    }

    pub fn execute(&self) -> Result<(), MagickError> {
        if self.input_files.is_empty() {
            return Err(wm_err!("no images defined")); // mimics imagemagick
        }
        crate::init::init();
        for (file_plan, output_file) in self.input_files.iter().zip(self.output_locations().iter())
        {
            let mut image = wm_try!(decode(&file_plan.location, file_plan.format));

            for operation in &file_plan.ops {
                operation.execute(&mut image)?;
            }

            encode::encode(
                &mut image,
                &output_file,
                self.output_format,
                &self.modifiers,
            )?;
        }

        Ok(())
    }

    fn output_locations(&self) -> Vec<Location> {
        if self.input_files.len() > 1 {
            if let Location::Path(output_file) = &self.output_file {
                let mut locations = Vec::new();
                for i in 1..=self.input_files.len() {
                    let suffix = OsString::from(format!("-{i}")); // indexing for output images starts at 1
                    let name =
                        insert_suffix_before_extension_in_path(output_file.as_os_str(), &suffix);
                    locations.push(Location::Path(PathBuf::from(name)))
                }
                return locations;
            }
        }
        vec![self.output_file.clone(); self.input_files.len()]
    }
}

/// Plan of operations for a single input file
#[derive(Debug, Default)]
pub struct FilePlan {
    pub location: Location,
    pub format: Option<ImageFormat>,
    pub ops: Vec<Operation>,
}

#[derive(Debug, Default)]
pub struct Modifiers {
    pub quality: Option<f64>,
    pub strip: Strip,
    pub identify_format: Option<IdentifyFormat>,
}

#[derive(Debug, Default, Copy, Clone)] // bools default to false
pub struct Strip {
    pub exif: bool,
    pub icc: bool,
    // TODO: XMP, etc: https://imagemagick.org/script/command-line-options.php#profile
}

impl Strip {
    pub fn set_all(&mut self, new_val: bool) {
        // enumerate the fields exhaustively so that the compiler complains if we miss any
        std::mem::swap(
            self,
            &mut Self {
                exif: new_val,
                icc: new_val,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_locations() {
        let plan = ExecutionPlan {
            output_file: Location::Path(PathBuf::from("out.gif")),
            input_files: vec![Default::default(), Default::default()],
            output_format: Some(ImageFormat::Jpeg), // Intentionally doesn't match the extension
            ..Default::default()
        };
        assert_eq!(
            plan.output_locations(),
            vec![
                Location::Path(PathBuf::from("out-1.gif")),
                Location::Path(PathBuf::from("out-2.gif")),
            ],
        );

        let plan = ExecutionPlan {
            output_file: Location::Path(PathBuf::from("no-extension")),
            input_files: vec![Default::default(), Default::default()],
            output_format: Some(ImageFormat::Jpeg),
            ..Default::default()
        };
        assert_eq!(
            plan.output_locations(),
            vec![
                Location::Path(PathBuf::from("no-extension-1")),
                Location::Path(PathBuf::from("no-extension-2")),
            ],
        );

        let plan = ExecutionPlan {
            output_file: Location::Stdio,
            input_files: vec![Default::default(), Default::default()],
            output_format: Some(ImageFormat::Jpeg),
            ..Default::default()
        };
        assert_eq!(
            plan.output_locations(),
            vec![Location::Stdio, Location::Stdio],
        );
    }
}
