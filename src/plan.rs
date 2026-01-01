use std::{
    ffi::{OsStr, OsString},
    path::PathBuf,
};

use crate::arg_parsers::{
    parse_numeric_arg, BlurGeometry, ChannelFormat, ColorModel, Colorspace, CropGeometry,
    FileFormat, GrayscaleMethod, IdentifyFormat, InputFileArg, Location, ResizeGeometry,
    UnsharpenGeometry,
};
use crate::args::{Arg, ArgSign, SignedArg};
use crate::decode::decode;
use crate::image::Image;
use crate::utils::filename::insert_suffix_before_extension_in_path;
use crate::{arg_parse_err::ArgParseErr, arg_parsers::Filter};
use crate::{encode, wm_err};
use crate::{
    error::MagickError,
    operations::Axis,
    operations::{Operation, RewriteOperation},
    wm_try,
};

/// Plan of operations for the whole run over multiple files
#[derive(Debug, Default)]
pub struct ExecutionPlan {
    /// Operations to be applied to ALL input files.
    ///
    /// We keep this list to construct a [`ExecutionStep::InputFile`].
    global_ops: Vec<Operation>,
    /// List of steps to execute on the image sequence itself once we start.
    execution: Vec<ExecutionStep>,
    /// Where to write the output image(s).
    output_file: Location,
    output_format: Option<FileFormat>,
    /// Current state of modifiers that apply to some operations. Modifiers are used by and applied
    /// to all later operations, when we add the operations to the `execution` list.
    modifiers: Modifiers,
}

/// Magick script occurs in a sequential order, manipulating a list of images.
///
/// We differentiate between different kinds of steps depending on their effect on the list to
/// simplify the model of possible result states. For instance, the simplest kind of operations
/// execute on each image individually and have no effect otherwise, these are implemented as
/// [`ExecutionStep::Map`]. Other operations take the whole list of images and produce one (not yet
/// implemented). Yet others may manipulate the list as a whole (not yet implemented). In input
/// files there are additional 'global' execution steps which apply to all input files, but we
/// store those in the [`FilePlan`] for the image when it the input execution step is created (see
/// [`ExecutionStep::InputFile`] for this particular subtlety).
#[derive(Debug)]
enum ExecutionStep {
    /// Apply an operation to all images in the current list of images (in imagemagick lingo, image
    /// stack). <https://imagemagick.org/script/command-line-processing.php#stack>
    ///
    /// Note that some options should be considered for each image individually such as
    /// percentage-qualified sizes in `-resize`.
    Map(Operation),
    /// Apply an operation that takes the whole current sequence, and produces new image(s).
    ///
    /// These operation may change the number of images in the sequence arbitrarily. They can not
    /// be easily parallelized over individual images. We hand over the whole sequence in a vector
    /// and expect it to be modified accordingly.
    Rewrite(RewriteOperation),
    /// Add a new image to the back of the sequence.
    ///
    ///
    /// A minor subtlety, in image magick the command `+write` can 'restore' an image to its
    /// 'original state'. This undoes all operations applied to the image so far but *not* the
    /// global operations that become part of the definition of this image. The image is restore to
    /// its state that resulted from this operation. Some formats, including sequences, have inline
    /// selection operations that modify the original definition as well. This is not yet
    /// implemented but will be important. For comparison, inspect:
    ///
    /// ```bash
    /// magick rose: -extract 20x20 +write 'null:' out.png # out is a 70x46 file
    /// magick -extract 20x20 rose: +write 'null:' out.png # out is a 20x20 file
    /// ```
    InputFile(FilePlan),
}

impl ExecutionPlan {
    pub fn apply_arg(
        &mut self,
        signed_arg: SignedArg,
        value: Option<&OsStr>,
    ) -> Result<(), MagickError> {
        let arg_string: &'static str = signed_arg.arg.into();
        if signed_arg.needs_value() != value.is_some() {
            return Err(wm_err!("argument requires a value: {arg_string}"));
        };

        self.apply_arg_inner(signed_arg, value).map_err(|arg_err| {
            wm_err!(
                "{}",
                arg_err.display_with_arg(arg_string, value.unwrap_or_default())
            )
        })?;

        Ok(())
    }

    /// Currently this can only fail due to argument parsing.
    /// Split into its own function due to lack of try{} blocks on stable Rust.
    fn apply_arg_inner(
        &mut self,
        signed_arg: SignedArg,
        value: Option<&OsStr>,
    ) -> Result<(), ArgParseErr> {
        match signed_arg.arg {
            Arg::AutoOrient => self.add_operation(Operation::AutoOrient),
            Arg::Colorspace => {
                let color = Colorspace::try_from(value.unwrap())?;
                self.modifiers.colorspace = Some(color);
            }
            Arg::Combine => {
                // Note: By experiment, and contrary to the documentation (ImageMagick 7.1.2-8),
                // the current `-channel` is ignored. The channel order is defined by the color
                // model alone.
                // Note: For `-combine` the current `-type` is used by default to choose a channel
                // model. If however more images are in the input than there are channels then _first_
                // the color model is adjusted to RGBA and then any still leftover channels are added
                // as meta channels. Meta channels are not yet supported by wondermagick so they will
                // be lost.
                // Note: The documentation lists a plus-style option that takes an additional
                // colorspace argument available only in `magick` script (not the old convert). The
                // handling of extra channels in this option is rather puzzling. Sometimes the alpha
                // channel is skipped, supplying five images to `+combine sRGB` makes red vertical
                // stripes appear in the ouptut (?). It may be memory corruption, at least the intent
                // is extremely unclear and not documented sufficiently. So we should not implement
                // that for now.
                if matches!(signed_arg.sign, ArgSign::Plus) {
                    // FIXME: We use the whole parser here despite only taking the model. The
                    // documentation does not say what happens to the transfer functions and all
                    // the other parts, i.e. interaction if a color profile is declared otherwise.
                    //
                    // There's a difference in chromaticity between these two:
                    //
                    // -colorspace rgb +combine lab
                    // -colorspace lab -combine
                    let model = Colorspace::try_from(value.unwrap())?.color;

                    self.add_rewrite(RewriteOperation::Combine {
                        color: self.color_type_for_model(model),
                        fallback_for_channel_count: false,
                    })?;
                } else {
                    let model = self
                        .modifiers
                        .colorspace
                        .map_or(ColorModel::Rgb, |sp| sp.color);

                    self.add_rewrite(RewriteOperation::Combine {
                        color: self.color_type_for_model(model),
                        fallback_for_channel_count: true,
                    })?;
                }
            }
            Arg::Crop => {
                self.add_operation(Operation::Crop(CropGeometry::try_from(value.unwrap())?))
            }
            Arg::Identify => {
                self.add_operation(Operation::Identify(self.modifiers.identify_format.clone()));
            }
            Arg::Blur => {
                self.add_operation(Operation::Blur(BlurGeometry::try_from(value.unwrap())?))
            }
            Arg::GaussianBlur => self.add_operation(Operation::GaussianBlur(
                BlurGeometry::try_from(value.unwrap())?,
            )),
            Arg::Grayscale => self.add_operation(Operation::Grayscale(GrayscaleMethod::try_from(
                value.unwrap(),
            )?)),
            Arg::Monochrome => self.add_operation(Operation::Monochrome),
            Arg::Negate => self.add_operation(Operation::Negate),
            Arg::Quality => self.modifiers.quality = Some(parse_numeric_arg(value.unwrap())?),
            Arg::Noise => self.add_operation(Operation::Noise),
            Arg::Resize => self.add_operation(Operation::Resize(
                ResizeGeometry::try_from(value.unwrap())?,
                self.modifiers.filter,
            )),
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
                self.add_operation(Operation::Thumbnail(
                    ResizeGeometry::try_from(value.unwrap())?,
                    self.modifiers.filter,
                ));
                // -thumbnail also strips all metadata except the ICC profile
                // Some docs state that it strips ICC profile also, but
                // https://usage.imagemagick.org/thumbnails/ says v6.5.4-7 onwards preserves them.
                self.modifiers.strip.set_all(true);
                self.modifiers.strip.icc = false;
            }
            Arg::Format => {
                self.modifiers.identify_format = Some(IdentifyFormat::try_from(value.unwrap())?)
            }
            Arg::Filter => self.modifiers.filter = Some(Filter::try_from(value.unwrap())?),
            Arg::Flip => self.add_operation(Operation::Flip(Axis::Vertical)),
            Arg::Flop => self.add_operation(Operation::Flip(Axis::Horizontal)),
            Arg::Unsharp => self.add_operation(Operation::Unsharpen(UnsharpenGeometry::try_from(
                value.unwrap(),
            )?)),
        };

        Ok(())
    }

    fn add_operation(&mut self, op: Operation) {
        // Operations such as -resize apply to all the files already listed (they map over the
        // current image sequence), but not subsequent ones,
        //
        // UNLESS they are specified before any of the files,
        // in which case they apply to all subsequent operations.
        //
        // FIXME: most operations are not allowed as global operations and they terminate the
        // global phase just like an input file. If used in global position they will then likely
        // error out due to the lack of input files.
        if self.execution.is_empty() {
            self.global_ops.push(op);
        } else {
            self.execution.push(ExecutionStep::Map(op));
        }
    }

    fn add_rewrite(&mut self, op: RewriteOperation) -> Result<(), ArgParseErr> {
        if self.execution.is_empty() {
            Err(ArgParseErr::with_msg(format_args!(
                "no input file for operation {op:?}"
            )))
        } else {
            self.execution.push(ExecutionStep::Rewrite(op));
            Ok(())
        }
    }

    pub fn add_input_file(&mut self, file: InputFileArg) {
        let mut file_plan = FilePlan {
            location: file.location,
            format: file.format,
            ops: self.global_ops.clone(),
        };

        // Operations are affected by Modifiers such as -format or -quality.
        // Their behavior is somewhat nontrivial.
        //
        // In imagemagick the modifier (usually) only applies if it comes BEFORE the operation it affects.
        // So `convert in.png -filter box -resize 100 out.png` uses box filter
        // but `convert in.png -resize 100 -filter box out.png` uses default filter.
        //
        // However! In `convert -resize 100 -filter box in.png out.png` the filter DOES apply.
        // This is because `-resize 100` comes before ALL filenames and is applied to all files,
        // and it reads the state of the modifiers at the point when the file is added.
        // This loop replicates this special behavior for global ops.
        for op in &mut file_plan.ops {
            op.apply_modifiers(&self.modifiers);
        }

        if let Some(file_mod) = file.read_mod {
            use crate::arg_parsers::ReadModifier::*;
            let op = match file_mod {
                Resize(geom) => Some(Operation::Resize(geom, None)),
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

        self.execution.push(ExecutionStep::InputFile(file_plan));
    }

    pub fn set_output_file(&mut self, file: Location, format: Option<FileFormat>) {
        self.output_file = file;
        self.output_format = format;
    }

    pub fn execute(&self) -> Result<(), MagickError> {
        // Operations before the first file become global. If no file was added we never switched
        // to execution on a sequence, so this is empty.
        if self.execution.is_empty() {
            return Err(wm_err!("no images defined")); // mimics imagemagick
        }

        crate::init::init();

        let mut sequence: Vec<Image> = vec![];
        for step in &self.execution {
            match step {
                ExecutionStep::Map(op) => {
                    for image in &mut sequence {
                        op.execute(image)?;
                    }
                }
                ExecutionStep::InputFile(file_plan) => {
                    let mut image = wm_try!(decode(&file_plan.location, file_plan.format));

                    for operation in &file_plan.ops {
                        operation.execute(&mut image)?;
                    }

                    sequence.push(image);
                }
                ExecutionStep::Rewrite(op) => {
                    op.execute(&mut sequence)?;
                }
            }
        }

        let output_locations = Self::output_locations(&self.output_file, &sequence);
        for (image, specific_location) in sequence.iter_mut().zip(output_locations) {
            encode::encode(
                image,
                &specific_location,
                self.output_format,
                &self.modifiers,
            )?;
        }

        Ok(())
    }

    /// There are actually two ways of encoding a sequence. If the format natively supports
    /// sequences (GIF, TIFF, etc) then the images are encoded as frames in that format. Otherwise
    /// the output location is treated as a pattern for multiple files (e.g. output-%d.png).
    ///
    /// FIXME: handle animated/sequence output
    fn output_locations(output: &Location, images: &[Image]) -> Vec<Location> {
        if images.len() > 1 {
            if let Location::Path(output_file) = output {
                let mut locations = Vec::new();
                for i in 1..=images.len() {
                    let suffix = OsString::from(format!("-{i}")); // indexing for output images starts at 1
                    let name =
                        insert_suffix_before_extension_in_path(output_file.as_os_str(), &suffix);
                    locations.push(Location::Path(PathBuf::from(name)))
                }
                return locations;
            }
        }

        vec![output.clone(); images.len()]
    }

    fn color_type_for_model(&self, model: ColorModel) -> image::ColorType {
        // FIXME: use the modifier of -depth and -storage-type
        // FIXME: apply -alpha settings that may force on and off
        model.with_channel_format(ChannelFormat::U8)
    }
}

/// Plan of operations for a single input file
#[derive(Debug, Default)]
pub struct FilePlan {
    pub location: Location,
    pub format: Option<FileFormat>,
    pub ops: Vec<Operation>,
}

#[derive(Debug, Default)]
pub struct Modifiers {
    pub quality: Option<f64>,
    pub strip: Strip,
    pub identify_format: Option<IdentifyFormat>,
    pub filter: Option<Filter>,
    pub colorspace: Option<Colorspace>,
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
        *self = Self {
            exif: new_val,
            icc: new_val,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use image::ImageFormat;

    #[test]
    fn test_output_locations() {
        let input = crate::image::InputProperties {
            filename: OsString::from("input.png"),
            color_type: image::ExtendedColorType::L16,
        };

        let two_sequence = [
            Image {
                format: None,
                exif: None,
                icc: None,
                pixels: image::DynamicImage::new_rgb8(1, 1),
                properties: input.clone(),
            },
            Image {
                format: Some(ImageFormat::Jpeg),
                exif: None,
                icc: None,
                pixels: image::DynamicImage::new_rgb8(1, 1),
                properties: input.clone(),
            },
        ];

        let outputs = ExecutionPlan::output_locations(
            &Location::Path(PathBuf::from("out.gif")),
            &two_sequence,
        );

        assert_eq!(
            outputs,
            vec![
                Location::Path(PathBuf::from("out-1.gif")),
                Location::Path(PathBuf::from("out-2.gif")),
            ],
        );

        let outputs = ExecutionPlan::output_locations(
            &Location::Path(PathBuf::from("no-extension")),
            &two_sequence,
        );

        assert_eq!(
            outputs,
            vec![
                Location::Path(PathBuf::from("no-extension-1")),
                Location::Path(PathBuf::from("no-extension-2")),
            ],
        );

        let outputs = ExecutionPlan::output_locations(&Location::Stdio, &two_sequence);

        assert_eq!(outputs, vec![Location::Stdio, Location::Stdio],);
    }
}
