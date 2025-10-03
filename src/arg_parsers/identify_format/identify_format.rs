use crate::{arg_parse_err::ArgParseErr, arg_parsers::identify_format::parser::parse};
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
        Ok(Self {
            template: Option::from(parse(s)?),
        })
    }
}
