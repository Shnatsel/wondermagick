use std::ffi::OsStr;

use current_platform::CURRENT_PLATFORM;
use strum::VariantArray;

use crate::args::Arg;

pub fn maybe_print_help_and_exit(bin_name: &str) {
    match std::env::args_os().nth(1) {
        None => print_help_and_exit(bin_name),
        Some(arg) => {
            if arg.as_os_str() == OsStr::new("--help") || arg.as_os_str() == OsStr::new("-help") {
                print_help_and_exit(bin_name)
            }
        }
    }
}

fn print_help_and_exit(bin_name: &str) -> ! {
    print_help(bin_name);
    std::process::exit(0);
}

fn print_help(bin_name: &str) {
    println!("Version: {}", version_string());
    println!("Copyright: (C) 2024-2025 WonderMagick contributors");
    println!("License: {}", env!("CARGO_PKG_LICENSE"));
    // TODO: "Features:"
    // TODO: "Delegates (built-in):"
    println!("Usage: {bin_name} [options ...] file [options ...] file");
    println!("");
    println!("Image Operators:");
    for arg in Arg::VARIANTS {
        let name: &'static str = arg.into();
        println!("  -{name:19} {}", arg.help_text());
    }
}

fn version_string() -> String {
    let cpu = CURRENT_PLATFORM.split('-').next().unwrap_or("unknown");
    let major = env!("CARGO_PKG_VERSION_MAJOR");
    let minor = env!("CARGO_PKG_VERSION_MINOR");
    let patch = env!("CARGO_PKG_VERSION_PATCH");
    let repo = env!("CARGO_PKG_REPOSITORY");

    format!("WonderMagick 6.{major}.{minor}-{patch} Q16 {cpu} 2050-01-01 {repo}")
}
