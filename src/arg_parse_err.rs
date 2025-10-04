use std::{ffi::OsStr, num::ParseFloatError};

/// Error reporting for argument parsing that mimics imagemagick.
/// Use `.display_with_arg()` to properly present this error.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ArgParseErr {
    pub message: Option<String>,
}

impl ArgParseErr {
    pub fn display_with_arg(&self, arg_name: &str, value: &OsStr) -> String {
        let value = value.to_string_lossy();
        // mimicking imagemagick: if there is a specific message, show it to the user,
        // otherwise simply echo the value the user has passed
        let message = if let Some(msg) = &self.message {
            msg.as_str()
        } else {
            &value
        };

        format!("invalid argument for option `{arg_name}': {message}")
    }

    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_msg(str: impl ToString) -> Self {
        let string = str.to_string();
        Self {
            message: Some(string),
        }
    }
}

impl From<ParseFloatError> for ArgParseErr {
    fn from(_value: ParseFloatError) -> Self {
        Self::new()
    }
}
