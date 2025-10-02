use crate::arg_parse_err::ArgParseErr;
use std::ffi::OsStr;

// https://imagemagick.org/script/escape.php

#[derive(Debug, Clone, PartialEq)]
pub enum Var {
    Width,
    Height,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Literal(String),
    Whitespace(usize),
    Var(Var),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct IdentifyFormat {
    pub template: Option<Vec<Token>>,
}

impl TryFrom<&std::ffi::OsStr> for IdentifyFormat {
    type Error = ArgParseErr;

    fn try_from(s: &OsStr) -> Result<Self, Self::Error> {
        Ok(Self {
            template: Option::from(vec![Token::Literal(s.to_string_lossy().into_owned())]),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identify_format_try_from() {
        let s = OsStr::new("just a sample literal string");
        let fmt = IdentifyFormat::try_from(s).unwrap();
        assert_eq!(
            fmt,
            IdentifyFormat {
                template: Some(vec![Token::Literal("just a sample literal string".into())])
            }
        );
    }
}
