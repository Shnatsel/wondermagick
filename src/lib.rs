//! `wondermagick` is not a library.
//! This interface is unstable and subject to change at any time.
//! Please use this documentation only if you are developing `wondermagick`.

#![forbid(unsafe_code)]

mod arg_parsers;
pub mod args;
pub mod decode;
mod encode;
mod encoders;
pub mod error;
mod filename_utils;
pub mod help;
pub mod image;
mod operations;
mod plan;
mod utils;
