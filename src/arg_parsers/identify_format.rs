use crate::arg_parse_err::ArgParseErr;
use std::ffi::OsStr;

// https://imagemagick.org/script/escape.php

#[derive(Debug, Clone, PartialEq)]
pub enum Var {
    Colorspace,
    CurrentImageHeightInPixels,
    CurrentImageWidthInPixels,
    ImageDepth,
    ImageFileFormat,
    ImageFilename,
    LayerCanvasPageGeometry,
    MagickFilename,
    OriginalImageSize,
    PageCanvasHeight,
    PageCanvasWidth,
    PageCanvasXOffset,
    PageCanvasYOffset,
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
        let mut tokens: Vec<Token> = Vec::new();
        let format_bytes = s.as_encoded_bytes();

        // TODO: Assumes there are only one-letter variables after '%', such as %w and %h.
        let mut state = ParseState::Initial;
        let mut literal_accumulator: Vec<u8> = Vec::new();
        let mut whitespace_count = 0;

        for char in format_bytes {
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
                    if !literal_accumulator.is_empty() {
                        // TODO: Can't assume valid UTF-8 here, the bytes come from `OsStr`
                        let literal = String::from_utf8(literal_accumulator.clone())
                            .map_err(|_e| ArgParseErr::new())?;
                        tokens.push(Token::Literal(literal));
                        literal_accumulator.clear();
                    }
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
                            b'G' => tokens.push(Token::Var(Var::OriginalImageSize)),
                            b'H' => tokens.push(Token::Var(Var::PageCanvasHeight)),
                            b'M' => tokens.push(Token::Var(Var::MagickFilename)),
                            b'W' => tokens.push(Token::Var(Var::PageCanvasWidth)),
                            b'X' => tokens.push(Token::Var(Var::PageCanvasXOffset)),
                            b'Y' => tokens.push(Token::Var(Var::PageCanvasYOffset)),
                            // TODO: 'c' is not true, there is no shorthand var for colorspace
                            b'c' => tokens.push(Token::Var(Var::Colorspace)),
                            b'g' => tokens.push(Token::Var(Var::LayerCanvasPageGeometry)),
                            b'h' => tokens.push(Token::Var(Var::CurrentImageHeightInPixels)),
                            b'i' => tokens.push(Token::Var(Var::ImageFilename)),
                            b'm' => tokens.push(Token::Var(Var::ImageFileFormat)),
                            b'w' => tokens.push(Token::Var(Var::CurrentImageWidthInPixels)),
                            b'z' => tokens.push(Token::Var(Var::ImageDepth)),
                            _ => {
                                return Err(ArgParseErr {
                                    message: Option::from(format!(
                                        "unknown shorthand variable '%{}'",
                                        *char as char
                                    )),
                                })
                            }
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
        assert_eq!(
            IdentifyFormat::try_from(OsStr::new("  ")).unwrap(),
            IdentifyFormat {
                template: Some(vec![Token::Whitespace(2)])
            }
        );
    }

    #[test]
    fn test_identify_format_try_from_with_literal() {
        assert_eq!(
            IdentifyFormat::try_from(OsStr::new("just a sample literal string")).unwrap(),
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
    // TODO: Cover all vars
    fn test_identify_format_try_from_with_shorthand_var() {
        assert_eq!(
            IdentifyFormat::try_from(OsStr::new("%w")).unwrap(),
            IdentifyFormat {
                template: Some(vec![Token::Var(Var::CurrentImageWidthInPixels)])
            }
        );
    }

    #[test]
    fn test_identify_format_try_from_with_shorthand_followed_by_letter() {
        assert_eq!(
            IdentifyFormat::try_from(OsStr::new("%wx%h")).unwrap(),
            IdentifyFormat {
                template: Some(vec![
                    Token::Var(Var::CurrentImageWidthInPixels),
                    Token::Literal("x".into()),
                    Token::Var(Var::CurrentImageHeightInPixels)
                ])
            }
        );
    }

    #[test]
    fn test_identify_format_try_from_with_unknown_shorthand() {
        assert_eq!(
            IdentifyFormat::try_from(OsStr::new("%a")),
            Err(ArgParseErr {
                message: Option::from(String::from("unknown shorthand variable '%a'"))
            })
        );
    }

    #[test]
    fn test_identify_format_try_from_with_non_ascii_literal() {
        assert_eq!(
            IdentifyFormat::try_from(OsStr::new("💪")).unwrap(),
            IdentifyFormat {
                template: Some(vec![Token::Literal("💪".into()),])
            }
        );
    }
}
