use pic_scale_safe::ResamplingFunction;

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

impl Filter {
    pub fn into_resize(self) -> ResamplingFunction {
        match self {
            Filter::Bartlett => todo!(),
            Filter::Blackman => todo!(),
            Filter::Bohman => todo!(),
            Filter::Box => ResamplingFunction::Box,
            Filter::Catrom => ResamplingFunction::CatmullRom,
            Filter::Cosine => todo!(),
            Filter::Cubic => todo!(),
            Filter::Gaussian => ResamplingFunction::Gaussian,
            Filter::Hamming => todo!(),
            Filter::Hann => todo!(),
            Filter::Hermite => todo!(),
            Filter::Jinc => todo!(),
            Filter::Kaiser => todo!(),
            Filter::Lagrange => ResamplingFunction::Lagrange3,
            Filter::Lanczos => ResamplingFunction::Lanczos3,
            Filter::Lanczos2 => ResamplingFunction::Lanczos2,
            Filter::Lanczos2Sharp => todo!(),
            Filter::LanczosRadius => todo!(),
            Filter::LanczosSharp => todo!(),
            Filter::Mitchell => todo!(),
            Filter::Parzen => todo!(),
            Filter::Point => ResamplingFunction::Nearest,
            Filter::Quadratic => todo!(),
            Filter::Robidoux => todo!(),
            Filter::RobidouxSharp => todo!(),
            Filter::Sinc => todo!(),
            Filter::SincFast => todo!(),
            Filter::Spline => todo!(),
            Filter::Triangle => ResamplingFunction::Bilinear,
            Filter::Welch => todo!(),
        }
    }
}
