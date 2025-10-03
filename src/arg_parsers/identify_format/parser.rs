use crate::arg_parse_err::ArgParseErr;
use crate::arg_parsers::Token;
use crate::arg_parsers::Var;
use std::ffi::OsStr;

enum ParseState {
    Initial,
    Literal,
    Whitespace,
    Var,
}

impl TryFrom<&u8> for Var {
    type Error = ArgParseErr;

    fn try_from(char: &u8) -> Result<Self, Self::Error> {
        match char {
            b'G' => Ok(Var::OriginalImageSize),
            b'H' => Ok(Var::PageCanvasHeight),
            b'M' => Ok(Var::MagickFilename),
            b'W' => Ok(Var::PageCanvasWidth),
            b'X' => Ok(Var::PageCanvasXOffset),
            b'Y' => Ok(Var::PageCanvasYOffset),
            // TODO: 'c' is not true, there is no shorthand var for colorspace
            b'c' => Ok(Var::Colorspace),
            b'g' => Ok(Var::LayerCanvasPageGeometry),
            b'h' => Ok(Var::CurrentImageHeightInPixels),
            b'i' => Ok(Var::ImageFilename),
            b'm' => Ok(Var::ImageFileFormat),
            b'w' => Ok(Var::CurrentImageWidthInPixels),
            b'z' => Ok(Var::ImageDepth),
            _ => {
                return Err(ArgParseErr {
                    message: Option::from(format!(
                        "unknown shorthand variable '%{}'",
                        *char as char
                    )),
                })
            }
        }
    }
}

pub fn parse(string: &OsStr) -> Result<Vec<Token>, ArgParseErr> {
    let mut tokens: Vec<Token> = Vec::new();
    let format_bytes = string.as_encoded_bytes();

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
                if whitespace_count > 0 {
                    tokens.push(Token::Whitespace(whitespace_count));
                    whitespace_count = 0;
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
                    tokens.push(Token::Var(Var::try_from(char)?));
                    state = ParseState::Initial;
                }
                ParseState::Literal => literal_accumulator.push(*char),
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
        ParseState::Var => return Err(ArgParseErr::new()),
        ParseState::Initial => {}
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_with_whitespace() {
        assert_eq!(parse(OsStr::new("  ")).unwrap(), vec![Token::Whitespace(2)]);
    }

    #[test]
    fn test_parse_with_literal() {
        assert_eq!(
            parse(OsStr::new("just a sample literal string")).unwrap(),
            vec![
                Token::Literal("just".into()),
                Token::Whitespace(1),
                Token::Literal("a".into()),
                Token::Whitespace(1),
                Token::Literal("sample".into()),
                Token::Whitespace(1),
                Token::Literal("literal".into()),
                Token::Whitespace(1),
                Token::Literal("string".into())
            ]
        );
    }

    #[test]
    // TODO: Cover all vars
    fn test_parse_with_shorthand_var() {
        assert_eq!(
            parse(OsStr::new("%w")).unwrap(),
            vec![Token::Var(Var::CurrentImageWidthInPixels)]
        );
    }

    #[test]
    fn test_parse_with_shorthand_followed_by_letter() {
        assert_eq!(
            parse(OsStr::new("%wx%h")).unwrap(),
            vec![
                Token::Var(Var::CurrentImageWidthInPixels),
                Token::Literal("x".into()),
                Token::Var(Var::CurrentImageHeightInPixels)
            ]
        );
    }

    #[test]
    fn test_parse_with_shorthand_followed_by_space() {
        assert_eq!(
            parse(OsStr::new("%w %h")).unwrap(),
            vec![
                Token::Var(Var::CurrentImageWidthInPixels),
                Token::Whitespace(1),
                Token::Var(Var::CurrentImageHeightInPixels),
            ]
        );
    }

    #[test]
    fn test_parse_with_unknown_shorthand() {
        assert_eq!(
            parse(OsStr::new("%a")),
            Err(ArgParseErr {
                message: Option::from(String::from("unknown shorthand variable '%a'"))
            })
        );
    }

    #[test]
    fn test_parse_with_non_ascii_literal() {
        assert_eq!(
            parse(OsStr::new("💪")).unwrap(),
            vec![Token::Literal("💪".into())]
        );
    }
}
