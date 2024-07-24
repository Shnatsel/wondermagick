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

#[macro_export]
macro_rules! wm_err {
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