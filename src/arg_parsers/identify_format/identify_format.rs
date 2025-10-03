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

impl TryFrom<&std::ffi::OsStr> for IdentifyFormat {
    type Error = ArgParseErr;

    fn try_from(s: &OsStr) -> Result<Self, Self::Error> {
        let tokens = crate::arg_parsers::identify_format::parser::parse(s)?;

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
    fn test_identify_format_try_from_with_shorthand_followed_by_space() {
        assert_eq!(
            IdentifyFormat::try_from(OsStr::new("%w %h")).unwrap(),
            IdentifyFormat {
                template: Some(vec![
                    Token::Var(Var::CurrentImageWidthInPixels),
                    Token::Whitespace(1),
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
