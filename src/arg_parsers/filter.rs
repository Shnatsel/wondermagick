use std::{ffi::OsStr, fmt::Display};

use pic_scale_safe::ResamplingFunction;

use crate::arg_parse_err::ArgParseErr;

#[derive(Copy, Clone, Eq, PartialEq, Debug, strum::EnumString, strum::IntoStaticStr)]
#[strum(ascii_case_insensitive)]
/// Represents imagemagick -filter options
pub enum Filter {
    // list obtained from 'convert -list filter'
    Bartlett,
    Blackman,
    Bohman,
    Box,
    Catrom,
    Cosine,
    Cubic,
    Gaussian,
    Hamming,
    Hann,
    Hermite,
    Jinc,
    Kaiser,
    Lagrange,
    Lanczos,
    Lanczos2,
    Lanczos2Sharp,
    LanczosRadius,
    LanczosSharp,
    Mitchell,
    Parzen,
    Point,
    Quadratic,
    Robidoux,
    RobidouxSharp,
    Sinc,
    SincFast,
    Spline,
    Triangle,
    Welch,
}

impl Display for Filter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let stringified: &'static str = self.into();
        f.write_str(stringified)
    }
}

impl TryFrom<&OsStr> for Filter {
    type Error = ArgParseErr;

    fn try_from(s: &OsStr) -> Result<Self, Self::Error> {
        if let Some(s_utf8) = s.to_str() {
            if let Ok(known_filter) = Self::try_from(s_utf8) {
                return Ok(known_filter);
            }
        }
        Err(ArgParseErr {
            message: Some(format!(
                "unrecognized image filter `{}'",
                s.to_string_lossy()
            )),
        })
    }
}

impl Filter {
    pub fn into_resize(self) -> ResamplingFunction {
        match self {
            Filter::Bartlett => ResamplingFunction::Bartlett,
            Filter::Blackman => ResamplingFunction::Blackman,
            Filter::Bohman => ResamplingFunction::Bohman,
            Filter::Box => ResamplingFunction::Box,
            Filter::Catrom => ResamplingFunction::CatmullRom,
            Filter::Cosine => ResamplingFunction::Hann, // Cosine is a raised cosine, similar to Hann
            Filter::Cubic => ResamplingFunction::Cubic,
            Filter::Gaussian => ResamplingFunction::Gaussian,
            Filter::Hamming => ResamplingFunction::Hamming,
            Filter::Hann => ResamplingFunction::Hann,
            Filter::Hermite => ResamplingFunction::Hermite,
            Filter::Jinc => ResamplingFunction::Lanczos3Jinc, // Jinc is often used with a Lanczos window
            Filter::Kaiser => ResamplingFunction::Kaiser,
            Filter::Lagrange => ResamplingFunction::Lagrange3,
            Filter::Lanczos => ResamplingFunction::Lanczos3,
            Filter::Lanczos2 => ResamplingFunction::Lanczos2,
            Filter::Lanczos2Sharp => ResamplingFunction::Lanczos2, // No direct sharp equivalent, fallback to Lanczos2
            Filter::LanczosRadius => ResamplingFunction::Lanczos3, // No direct equivalent, fallback to Lanczos3
            Filter::LanczosSharp => ResamplingFunction::Lanczos3, // No direct sharp equivalent, fallback to Lanczos3
            Filter::Mitchell => ResamplingFunction::MitchellNetravalli,
            Filter::Parzen => ResamplingFunction::BSpline, // Parzen is a cubic B-spline
            Filter::Point => ResamplingFunction::Nearest,
            Filter::Quadratic => ResamplingFunction::Quadric,
            Filter::Robidoux => ResamplingFunction::Robidoux,
            Filter::RobidouxSharp => ResamplingFunction::RobidouxSharp,
            Filter::Sinc => ResamplingFunction::Lanczos3, // Sinc is typically windowed, Lanczos is a good default
            Filter::SincFast => ResamplingFunction::Lanczos2, // A faster, likely lower-quality Sinc, Lanczos2 is a reasonable approximation
            Filter::Spline => ResamplingFunction::BSpline,
            Filter::Triangle => ResamplingFunction::Bilinear,
            Filter::Welch => ResamplingFunction::Welch,
        }
    }
}
