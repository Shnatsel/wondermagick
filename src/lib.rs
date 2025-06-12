//! `wondermagick` is not a library.
//! This interface is unstable and subject to change at any time.
//! Please use this documentation only if you are developing `wondermagick`.

#![forbid(unsafe_code)]

mod arg_parsers;
pub mod args;
pub mod decode;
pub mod error;
pub mod image;
mod filename_utils;
pub mod help;
mod operations;
mod plan;
mod utils;
