use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::{error::MagickError, wm_err};

use super::{Geometry, ResizeGeometry};

pub struct InputFileArg {
    path: PathBuf,
    //format: Option<String>, // TODO: turn into an enum and enable
    read_mod: Option<ReadModifier>,
}

/// The action to be taken upon loading the image.
/// `convert` accepts any single one of: frame selection, resize, or crop.
///
/// See <https://imagemagick.org/Usage/files/#read_mods> for details.
/// I've also verified it behaves according to the documentation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReadModifier {
    Resize(ResizeGeometry),
    Crop(LoadCropGeometry),
    // TODO: frame selection.
}

impl TryFrom<&OsStr> for ReadModifier {
    type Error = MagickError;

    fn try_from(s: &OsStr) -> Result<Self, Self::Error> {
        if !s.is_ascii() {
            return Err(wm_err!("invalid read modifier: {}", s.to_string_lossy()));
        }

        let ascii = s.as_encoded_bytes();
        let x_count = ascii.iter().copied().filter(|c| *c == b'x').take(2).count();
        let plus_count = ascii.iter().copied().filter(|c| *c == b'+').take(3).count();

        if x_count == 1 && plus_count == 0 {
            Ok(Self::Resize(ResizeGeometry::try_from(s)?))
        } else if x_count == 1 && plus_count == 2 {
            Ok(Self::Crop(LoadCropGeometry::try_from(s)?))
        } else {
            Err(wm_err!("frame selection is not yet supported")) // TODO: support [0] at least
        }
    }
}

/// On loading only a subset of crop geometry specification is supported:
/// it *must* be in the form AxB+C+D, see
/// https://imagemagick.org/Usage/files/#read_mods
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LoadCropGeometry {
    pub width: u32,
    pub height: u32,
    pub xoffset: u32,
    pub yoffset: u32,
}

impl FromStr for LoadCropGeometry {
    type Err = MagickError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(OsStr::new(s))
    }
}

impl TryFrom<&OsStr> for LoadCropGeometry {
    type Error = MagickError;

    fn try_from(s: &OsStr) -> Result<Self, Self::Error> {
        let geom = Geometry::try_from(s)?;

        let convert_field = |field: Option<f64>| -> Result<u32, MagickError> {
            let f =
                field.ok_or_else(|| wm_err!("invalid crop geometry: {}", s.to_string_lossy()))?;
            if f.is_sign_negative() {
                Err(wm_err!("invalid crop geometry: {}", s.to_string_lossy()))
            } else {
                Ok(f as u32)
            }
        };

        Ok(Self {
            width: convert_field(geom.width)?,
            height: convert_field(geom.height)?,
            xoffset: convert_field(geom.xoffset)?,
            yoffset: convert_field(geom.yoffset)?,
        })
    }
}

fn file_exists(path: &Path) -> bool {
    // This is wrapped into our own function so that we could mock it for unit tests
    path.is_file()
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::LoadCropGeometry;

    #[test]
    fn load_crop_geometry() {
        // only a basic smoke test because the underlying geometry parser is well tested already
        let expected = LoadCropGeometry {
            width: 1,
            height: 2,
            xoffset: 3,
            yoffset: 4,
        };
        let parsed = LoadCropGeometry::from_str("1x2+3+4").unwrap();
        assert_eq!(expected, parsed);
    }
}
