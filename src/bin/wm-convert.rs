use std::error::Error;
use wondermagick::{args, decode::decode, help};

fn main() {
    if let Err(e) = real_main() {
        eprintln!("{}", e);
    }
}

fn real_main() -> Result<(), Box<dyn Error>> {
    help::maybe_print_help_and_exit(env!("CARGO_BIN_NAME"));
    let arguments: Vec<_> = std::env::args_os().collect();
    let plan = args::parse_args(arguments)?;

    // TODO: handle multiple images
    let file_plan = plan.input_files.first().unwrap();
    let mut image = decode(&file_plan.filename, None)?;

    for operation in &file_plan.ops {
        operation.execute(&mut image)?;
    }

    image.save(plan.output_file)?;
    Ok(())
}
