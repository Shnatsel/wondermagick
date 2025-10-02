use crate::arg_parse_err::ArgParseErr;
use crate::wm_err;
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

enum ParseState {
    Initial,
    Literal,
    Whitespace,
    Var,
}

impl TryFrom<&std::ffi::OsStr> for IdentifyFormat {
    type Error = ArgParseErr;

    fn try_from(s: &OsStr) -> Result<Self, Self::Error> {
        if !s.is_ascii() {
            return Err(ArgParseErr::new());
        }

        let mut tokens: Vec<Token> = Vec::new();
        let ascii = s.as_encoded_bytes();

        // TODO: Assumes there are only one-letter variables after '%', such as %w and %h.
        let mut state = ParseState::Initial;
        let mut literal_accumulator: Vec<u8> = Vec::new();
        let mut whitespace_count = 0;

        for char in ascii {
            match char {
                b' ' => {
                    state = ParseState::Whitespace;
                    whitespace_count += 1;
                    if !literal_accumulator.is_empty() {
                        let literal = String::from_utf8(literal_accumulator.clone())
                            .map_err(|_e| ArgParseErr::new())?;
                        tokens.push(Token::Literal(literal));
                        literal_accumulator.clear();
                    }
                }
                b'%' => {
                    state = ParseState::Var;
                }
                _ => match state {
                    ParseState::Initial => {
                        state = ParseState::Literal;
                        literal_accumulator.push(*char);
                    }
                    ParseState::Whitespace => {
                        if whitespace_count > 0 {
                            tokens.push(Token::Whitespace(whitespace_count));
                            whitespace_count = 0;
                        }
                        state = ParseState::Literal;
                        literal_accumulator.push(*char);
                    }
                    ParseState::Var => {
                        match char {
                            b'w' => tokens.push(Token::Var(Var::Width)),
                            b'h' => tokens.push(Token::Var(Var::Height)),
                            _ => return Err(ArgParseErr::new()),
                        }
                        state = ParseState::Initial;
                    }
                    ParseState::Literal => {
                        literal_accumulator.push(*char);
                    }
                },
            }
        }

        match state {
            ParseState::Literal => {
                if !literal_accumulator.is_empty() {
                    let literal = String::from_utf8(literal_accumulator.clone())
                        .map_err(|_e| ArgParseErr::new())?;
                    tokens.push(Token::Literal(literal));
                }
            }
            ParseState::Whitespace => {
                if whitespace_count > 0 {
                    tokens.push(Token::Whitespace(whitespace_count));
                }
            }
            ParseState::Var => {
                return Err(ArgParseErr::new());
            }
            ParseState::Initial => {}
        }

        Ok(Self {
            template: Option::from(tokens),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identify_format_try_from_with_whitespace() {
        let s = OsStr::new("  ");
        let fmt = IdentifyFormat::try_from(s).unwrap();
        assert_eq!(
            fmt,
            IdentifyFormat {
                template: Some(vec![Token::Whitespace(2)])
            }
        );
    }

    #[test]
    fn test_identify_format_try_from_with_literal() {
        let s = OsStr::new("just a sample literal string");
        let fmt = IdentifyFormat::try_from(s).unwrap();
        assert_eq!(
            fmt,
            IdentifyFormat {
                template: Some(vec![
                    Token::Literal("just".into()),
                    Token::Whitespace(1),
                    Token::Literal("a".into()),
                    Token::Whitespace(1),
                    Token::Literal("sample".into()),
                    Token::Whitespace(1),
                    Token::Literal("literal".into()),
                    Token::Whitespace(1),
                    Token::Literal("string".into())
                ])
            }
        );
    }

    #[test]
    fn test_identify_format_try_from_with_replacement_var() {
        let s = OsStr::new("%w");
        let fmt = IdentifyFormat::try_from(s).unwrap();
        assert_eq!(
            fmt,
            IdentifyFormat {
                template: Some(vec![Token::Var(Var::Width)])
            }
        );
    }
}
