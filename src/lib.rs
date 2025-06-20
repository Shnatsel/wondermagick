//! `wondermagick` is not a library.
//! This interface is unstable and subject to change at any time.
//! Please use this documentation only if you are developing `wondermagick`.

#![forbid(unsafe_code)]

#[cfg(feature = "hardened_malloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

mod arg_parse_err;
mod arg_parsers;
pub mod args;
mod decode;
mod encode;
mod encoders;
pub mod error;
mod filename_utils;
pub mod help;
mod image;
mod operations;
mod plan;
mod utils;
