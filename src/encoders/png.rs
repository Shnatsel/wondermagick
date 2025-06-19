use std::io::Write;

use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::ImageEncoder;

use crate::{error::MagickError, image::Image, wm_try};

pub fn encode<W: Write>(image: &Image, writer: &mut W,  quality: Option<u8>) -> Result<(), MagickError> { 
    let (compression, filter) = quality_to_compression_parameters(quality);
    let mut encoder = PngEncoder::new_with_quality(writer, compression, filter);
    if let Some(icc) = image.icc.clone() {
        let _ = encoder.set_icc_profile(icc); // ignore UnsupportedError
    };
    Ok(wm_try!(image.pixels.write_with_encoder(encoder)))
}

// for documentation on conversion of quality to encoding parameters see
// https://www.imagemagick.org/script/command-line-options.php#quality
fn quality_to_compression_parameters(quality: Option<u8>) -> (CompressionType, FilterType) {
    if let Some(quality) = quality {
        // TODO: correct quality mapping is blocked on upstream issue:
        // https://github.com/image-rs/image/issues/2495
        let compression = match quality / 10 {
            0..=2 => CompressionType::Fast,
            3..=7 => CompressionType::Default,
            8.. => CompressionType::Best,
        };
        let filter = match quality % 10 {
            0 => FilterType::NoFilter,
            1 => FilterType::Sub,
            2 => FilterType::Up,
            3 => FilterType::Avg,
            4 => FilterType::Paeth,
            // 7 is documented as MNG-only, in practice maps to 5 or 6?
            5..=7 => FilterType::Adaptive,
            // filters 8 and 9 override compression level selection
            8 => return (CompressionType::Fast, FilterType::Adaptive),
            // imagemagick uses filter=None here, but our Fast mode needs filtering
            // to deliver reasonable compression, so use the fastest filter instead
            9 => return (CompressionType::Fast, FilterType::Up),
            10.. => unreachable!(),
        };

        if filter == FilterType::NoFilter && compression == CompressionType::Fast {
            // CompressionType::Fast needs filtering for a reasonable compression ratio.
            // When using it, use the fastest filter instead of no filter at all.
            (CompressionType::Fast, FilterType::Up)
        } else {
            (compression, filter)
        }
    } else {
        (CompressionType::Default, FilterType::Adaptive)
    }
}