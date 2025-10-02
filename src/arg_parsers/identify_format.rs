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

    fn try_from(_s: &OsStr) -> Result<Self, Self::Error> {
        Ok(Self {
            template: Option::from(vec![Token::Literal("Hello world".into())]),
        })
    }
}
