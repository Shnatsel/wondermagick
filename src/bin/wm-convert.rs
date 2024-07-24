use image::ImageReader;
use std::error::Error;
use wondermagick::args;

fn main() {
    if let Err(e) = real_main() {
        eprintln!("{}", e);
    }
}

fn real_main() -> Result<(), Box<dyn Error>> {
    // TODO: handle multiple images
    args::maybe_print_help();
    let arguments: Vec<_> = std::env::args_os().collect();
    let plan = args::parse_args(arguments)?;

    // TODO: handle multiple images
    let file_plan = plan.input_files.first().unwrap();
    let mut image = ImageReader::open(&file_plan.filename)?
        .with_guessed_format()?
        .decode()?;

    for operation in &file_plan.ops {
        operation.execute(&mut image);
    }

    image.save(plan.output_file)?;
    Ok(())
}