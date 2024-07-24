use std::fmt::{Debug, Display};
pub struct MagickError(pub String);

impl Display for MagickError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl Debug for MagickError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("MagickError").field(&self.0).finish()
    }
}

impl std::error::Error for MagickError {}

/// Similar to `format!`, but returns a `MagickError` and also records the source code location where it was called.
/// We use it to imitate the structure of imagemagick's error messages.
#[macro_export]
macro_rules! wm_err {
    // TODO: consider also recording function path: https://stackoverflow.com/a/40234666/585725
    ($msg:expr) => {
        MagickError(format!(
            "wondermagick: {} @ {}:{}:{}",
            $msg,
            file!(),
            line!(),
            column!()
        ))
    };
}

/// Similar to the `try!` macro and the `?` operator, but also
/// records the source code location where it was called.
#[macro_export]
macro_rules! wm_try {
    ($expr:expr $(,)?) => {
        match $expr {
            std::result::Result::Ok(val) => val,
            std::result::Result::Err(err) => {
                return std::result::Result::Err(wm_err!(err));
            }
        }
    };
}