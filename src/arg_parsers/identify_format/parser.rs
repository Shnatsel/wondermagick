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
            _ => Err(ArgParseErr {
                message: Option::from(format!("unknown shorthand variable '%{}'", *char as char)),
            }),
        }
    }
}

struct Parser {
    state: ParseState,
    literal_accumulator: Vec<u8>,
    whitespace_count: usize,
    tokens: Vec<Token>,
}

impl Parser {
    fn new() -> Self {
        Self {
            state: ParseState::Initial,
            literal_accumulator: Vec::new(),
            whitespace_count: 0,
            tokens: Vec::new(),
        }
    }

    fn start_whitespace(&mut self) {
        self.state = ParseState::Whitespace;
        self.whitespace_count += 1;
    }

    fn start_literal(&mut self, char: &u8) {
        self.state = ParseState::Literal;
        self.literal_accumulator.push(*char);
    }

    fn try_finish_literal(&mut self) -> Result<(), ArgParseErr> {
        if !self.literal_accumulator.is_empty() {
            let literal = String::from_utf8(self.literal_accumulator.clone())
                .map_err(|_e| ArgParseErr::new())?;
            self.tokens.push(Token::Literal(literal));
            self.literal_accumulator.clear();
        }
        Ok(())
    }

    // TODO: Assumes there are only one-letter variables after '%', such as %w and %h.
    fn try_finish_var(&mut self, char: &u8) -> Result<(), ArgParseErr> {
        self.tokens.push(Token::Var(Var::try_from(char)?));
        self.state = ParseState::Initial;
        Ok(())
    }

    fn finish_whitespace(&mut self) {
        if self.whitespace_count > 0 {
            self.tokens.push(Token::Whitespace(self.whitespace_count));
            self.whitespace_count = 0;
        }
    }

    fn try_parse_char(&mut self, char: &u8) -> Result<(), ArgParseErr> {
        match char {
            b' ' => {
                self.try_finish_literal()?;
                self.start_whitespace();
            }
            b'%' => {
                if self
                    .literal_accumulator
                    .last()
                    .map_or(false, |&c| c.is_ascii_digit())
                {
                    self.literal_accumulator.push(*char);
                } else {
                    self.try_finish_literal()?;
                    self.finish_whitespace();
                    self.state = ParseState::Var;
                }
            }
            _ => match self.state {
                ParseState::Initial => self.start_literal(char),
                ParseState::Whitespace => {
                    self.finish_whitespace();
                    self.start_literal(char);
                }
                ParseState::Var => self.try_finish_var(char)?,
                ParseState::Literal => self.literal_accumulator.push(*char),
            },
        }
        Ok(())
    }

    fn try_finish(&mut self) -> Result<(), ArgParseErr> {
        match self.state {
            ParseState::Literal => self.try_finish_literal()?,
            ParseState::Whitespace => self.finish_whitespace(),
            ParseState::Var => return Err(ArgParseErr::new()),
            ParseState::Initial => {}
        }
        Ok(())
    }
}

pub fn parse(string: &OsStr) -> Result<Vec<Token>, ArgParseErr> {
    let mut parser = Parser::new();
    let format_bytes = string.to_string_lossy().as_bytes().to_owned();

    for char in format_bytes {
        parser.try_parse_char(&char)?;
    }

    parser.try_finish()?;

    Ok(parser.tokens)
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
    fn test_parse_with_shorthand_var() {
        let cases = [
            (OsStr::new("%w"), Var::CurrentImageWidthInPixels),
            (OsStr::new("%G"), Var::OriginalImageSize),
            (OsStr::new("%H"), Var::PageCanvasHeight),
            (OsStr::new("%M"), Var::MagickFilename),
            (OsStr::new("%W"), Var::PageCanvasWidth),
            (OsStr::new("%X"), Var::PageCanvasXOffset),
            (OsStr::new("%Y"), Var::PageCanvasYOffset),
            (OsStr::new("%c"), Var::Colorspace),
            (OsStr::new("%g"), Var::LayerCanvasPageGeometry),
            (OsStr::new("%h"), Var::CurrentImageHeightInPixels),
            (OsStr::new("%i"), Var::ImageFilename),
            (OsStr::new("%m"), Var::ImageFileFormat),
            (OsStr::new("%w"), Var::CurrentImageWidthInPixels),
            (OsStr::new("%z"), Var::ImageDepth),
        ];

        for (string, var) in cases {
            assert_eq!(parse(string).unwrap(), vec![Token::Var(var)]);
        }
    }

    // From https://imagemagick.org/script/escape.php:
    // WARNING: short form percent escapes are NOT performed when the percent
    // is after a number. For example, 10%x10 does not expand the %x as a percent escape.
    // If you specifically want to expand the 'x', use the long form which overrides this
    // special case. EG: 10%[x]10.
    #[test]
    fn test_parse_with_shorthand_after_number() {
        assert_eq!(
            parse(OsStr::new("10%x10")).unwrap(),
            vec![Token::Literal("10%x10".into())]
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
