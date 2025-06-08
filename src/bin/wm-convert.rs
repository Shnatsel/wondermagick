use wondermagick::{args, error::MagickError, help};

fn main() {
    if let Err(e) = real_main() {
        eprintln!("{}", e);
    }
}

fn real_main() -> Result<(), MagickError> {
    help::maybe_print_help_and_exit(env!("CARGO_BIN_NAME"));
    let arguments: Vec<_> = std::env::args_os().collect();
    let plan = args::parse_args(arguments)?;
    plan.execute()
}
