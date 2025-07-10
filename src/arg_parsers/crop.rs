use std::ffi::OsStr;

use crate::{arg_parse_err::ArgParseErr, arg_parsers::ExtGeometry};

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct CropArea {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub xoffset: Option<i32>,
    pub yoffset: Option<i32>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CropGeometry {
    pub area: CropArea,
    pub slice_into_many: bool,
    pub repage: bool,
    pub percentage_mode: bool,
}

impl TryFrom<&OsStr> for CropGeometry {
    type Error = ArgParseErr;

    fn try_from(s: &OsStr) -> Result<Self, Self::Error> {
        if !s.is_ascii() {
            return Err(ArgParseErr::new());
        }

        let geom_ext = ExtGeometry::try_from(s)?;

        // imagemagick slices the image into many smaller images if you use "-crop 50x50", you need "-crop 50x50+0" for a single image.
        // it's not possible to express a yoffset without specifying an xoffset, so no need to check both.
        let slice_into_many = geom_ext.geom.xoffset.is_none();

        let flags = geom_ext.flags;
        let repage = flags.exclamation;
        let percentage_mode = flags.percent;

        // TODO: bug-compatibility with the weird behavior for these technically accepted flags
        // let area_mode = flags.at;
        // let cover_mode = flags.caret;
        // let only_enlarge = flags.less_than;
        // let only_shrink = flags.greater_than;

        let geom = geom_ext.geom;
        let area = CropArea {
            width: geom.width.map(|f| f.round() as u32),
            height: geom.height.map(|f| f.round() as u32),
            xoffset: geom.xoffset.map(|f| f.round() as i32),
            yoffset: geom.yoffset.map(|f| f.round() as i32),
        };

        Ok(Self {
            area,
            slice_into_many,
            repage,
            percentage_mode,
        })
    }
}
