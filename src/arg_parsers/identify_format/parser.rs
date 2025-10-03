use crate::arg_parse_err::ArgParseErr;
use crate::arg_parsers::Token;
use crate::arg_parsers::Var;
use std::ffi::OsStr;

pub enum ParseState {
    Initial,
    Literal,
    Whitespace,
    Var,
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
