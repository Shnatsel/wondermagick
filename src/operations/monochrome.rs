use crate::{arg_parse_err::ArgParseErr, error::MagickError, image::Image};
use image::{DynamicImage, GrayImage, Luma};

/// Runtime configuration for the monochrome dithering operator.
/// All fields can be set from the command line or left at defaults.
#[derive(Debug, Clone, PartialEq)]
pub struct MonochromeConfig {
    // Scales how much the blue noise texture shifts the black/white
    // decision point per pixel.
    //   0.0: No noise — clean mathematical dither bands
    //  20.0 to 60.0: Subtle breakup of banding artifacts
    //  80.0 to 140.0: Film-grain sweet spot (default 100)
    // 150.0 to 255.0: Heavy noise, coarse texture
    pub dither_noise: f32,

    // Controls how far below the dark centroid the dither band extends.
    // 0.0 = dither starts exactly at the dark centroid (IM6 default, hard black threshold)
    // 0.5 = extends halfway from the dark centroid toward pure black
    // 1.0 = extends all the way to pure black (maximum shadow graduation)
    pub dither_band_shadow_expansion: f32,

    // Controls how far above the light centroid the dither band extends.
    // 0.0 = dither ends exactly at the light centroid (IM6 default, hard white threshold)
    // 0.5 = extends halfway from the light centroid toward pure white
    // 1.0 = extends all the way to pure white (maximum highlight graduation)
    pub dither_band_highlight_expansion: f32,

    // Pre-dither brightness adjustment, applied to the grayscale image.
    // Additive shift scaled to the 0–255 range.
    // Range: -100.0 to 100.0 (0.0 = no change)
    // Positive values lighten the image, negative values darken it.
    pub brightness: f32,

    // Pre-dither contrast adjustment, applied to the grayscale image.
    // Multiplicative scale around the midpoint
    // Range: -100.0 to 100.0 (0.0 = no change)
    // Positive values increase contrast (expand away from midpoint),
    // negative values decrease it (compress toward midpoint).
    pub contrast: f32,

    // Pre-dither gamma adjustment, applied to the grayscale image.
    // Power curve on normalized pixel values.
    // Range: -100.0 to 100.0 (0.0 = no change)
    // Negative values brighten midtones (gamma < 1), positive darken them (gamma > 1).
    pub gamma: f32,
}

impl Default for MonochromeConfig {
    fn default() -> Self {
        Self {
            dither_noise: 100.0,
            dither_band_shadow_expansion: 0.0,
            dither_band_highlight_expansion: 0.0,
            brightness: 0.0,
            contrast: 0.0,
            gamma: 0.0,
        }
    }
}

impl MonochromeConfig {
    /// Parse from a comma-separated string of six floats, or the literal "default".
    /// Format: "dither_noise,shadow_expansion,highlight_expansion,brightness,contrast,gamma"
    /// Example: "100,0,0.5,10,-5,0" or "default"
    pub fn parse_arg(s: &str) -> Result<Self, ArgParseErr> {
        let s = s.trim();
        if s.eq_ignore_ascii_case("default") {
            return Ok(Self::default());
        }

        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() != 6 {
            return Err(ArgParseErr::with_msg(
                "monochrome requires 'default' or exactly 6 comma-separated values: \
                 noise,shadow_expansion,highlight_expansion,brightness,contrast,gamma",
            ));
        }

        let parse = |i: usize, name: &str| -> Result<f32, ArgParseErr> {
            parts[i]
                .trim()
                .parse::<f32>()
                .map_err(|_| ArgParseErr::with_msg(format_args!("invalid {name} value")))
        };

        let range = |val: f32, lo: f32, hi: f32, name: &str| -> Result<f32, ArgParseErr> {
            if val < lo || val > hi {
                Err(ArgParseErr::with_msg(format_args!(
                    "{name} must be between {lo} and {hi}, got {val}"
                )))
            } else {
                Ok(val)
            }
        };

        let dither_noise = range(parse(0, "dither_noise")?, 0.0, 255.0, "dither_noise")?;
        let dither_band_shadow_expansion = range(
            parse(1, "dither_band_shadow_expansion")?,
            0.0,
            1.0,
            "dither_band_shadow_expansion",
        )?;
        let dither_band_highlight_expansion = range(
            parse(2, "dither_band_highlight_expansion")?,
            0.0,
            1.0,
            "dither_band_highlight_expansion",
        )?;
        let brightness = range(parse(3, "brightness")?, -100.0, 100.0, "brightness")?;
        let contrast = range(parse(4, "contrast")?, -100.0, 100.0, "contrast")?;
        let gamma = range(parse(5, "gamma")?, -100.0, 100.0, "gamma")?;

        Ok(Self {
            dither_noise,
            dither_band_shadow_expansion,
            dither_band_highlight_expansion,
            brightness,
            contrast,
            gamma,
        })
    }
}

pub fn monochrome(image: &mut Image, config: &MonochromeConfig) -> Result<(), MagickError> {
    let mut grayscaled = image.pixels.to_luma8();
    adjust_colors(&mut grayscaled, config);
    remap_to_dither_band(&mut grayscaled, config);
    apply_blue_noise_scatter(&mut grayscaled, config);
    image.pixels = DynamicImage::ImageLuma8(grayscaled);
    Ok(())
}

/// Apply brightness, contrast, and gamma adjustments to a grayscale image.
///
/// Follows the gmic `adjust_colors` conventions:
///  1. Brightness: additive shift (percentage of full range)
///  2. Contrast: multiplicative scale around the midpoint
///  3. Gamma: power-law curve on normalized values
///
/// All three parameters use a -100..100 range with 0 meaning no change.
/// Order of operations matches gmic: brightness → contrast → gamma.
fn adjust_colors(image: &mut GrayImage, config: &MonochromeConfig) {
    if config.brightness == 0.0 && config.contrast == 0.0 && config.gamma == 0.0 {
        return;
    }

    // Contrast factor: map -100..100 to a multiplier.
    // At 0 → 1.0 (no change), at 100 → ~3.0, at -100 → ~0.0
    let contrast_factor = ((config.contrast + 100.0) / 100.0).powi(2);

    // Gamma exponent: map -100..100 to a power value.
    // At 0 → 1.0 (no change), negative → <1 (brighten midtones),
    // positive → >1 (darken midtones)
    let gamma_exp = if config.gamma == 0.0 {
        1.0
    } else {
        // Map -100..100 → exponent range ~0.1..10.0 via power of 10
        // gmic uses: gamma_val = 10^(gamma_pct / 100)
        // so -100 → 0.1 (strong brighten), 0 → 1.0, 100 → 10.0 (strong darken)
        10.0_f32.powf(config.gamma / 100.0)
    };

    // Brightness offset: map -100..100 to -255..255
    let brightness_offset = config.brightness * 255.0 / 100.0;

    for pixel in image.pixels_mut() {
        let mut v = pixel.0[0] as f32;

        // 1. Brightness: additive shift
        v += brightness_offset;

        // 2. Contrast: scale around midpoint (127.5)
        v = 127.5 + (v - 127.5) * contrast_factor;

        // 3. Gamma: power curve on normalized value
        if gamma_exp != 1.0 {
            let normalized = (v / 255.0).clamp(0.0, 1.0);
            v = normalized.powf(gamma_exp) * 255.0;
        }

        pixel.0[0] = v.clamp(0.0, 255.0) as u8;
    }
}

/// Two-color quantization matching ImageMagick 6's `-monochrome` behavior.
///
/// IM6 does not dither between pure 0 and 255. Instead it finds two
/// representative gray levels (cluster centroids) from the image histogram,
/// then dithers only between those. Pixels darker than the dark centroid
/// are hard-thresholded to black, pixels brighter than the light centroid
/// are hard-thresholded to white, and only the tones in between get dithered.
///
/// The dither_band_shadow_expansion and dither_band_highlight_expansion
/// fields let you widen the dither range beyond the centroids.
/// At 0.0 you get the IM6-like hard threshold at the extremes.
/// At 1.0 the dither band reaches all the way to 0 or 255, giving
/// full graduation in shadows or highlights respectively.
fn remap_to_dither_band(image: &mut GrayImage, config: &MonochromeConfig) {
    // Build histogram
    let mut histogram = [0u32; 256];
    for pixel in image.pixels() {
        histogram[pixel.0[0] as usize] += 1;
    }

    // Find two centroids via k-means on the 1-D histogram
    let (c_dark, c_light) = two_color_centroids(&histogram);

    // Expand the dither band beyond the centroids based on tuning constants.
    // shadow_expansion=0 → band starts at c_dark, =1 → band starts at 0
    // highlight_expansion=0 → band ends at c_light, =1 → band ends at 255
    let band_low = c_dark - (c_dark * config.dither_band_shadow_expansion);
    let band_high = c_light + ((255.0 - c_light) * config.dither_band_highlight_expansion);

    // Remap: values at or below band_low → 0, at or above band_high → 255,
    // values in between are linearly interpolated across 0..=255.
    // This gives the ditherer a full 0–255 working range but only for
    // the tones that fall within the expanded dither band.
    let range = band_high - band_low;
    if range < 1.0 {
        return;
    }

    for pixel in image.pixels_mut() {
        let v = pixel.0[0] as f32;
        let remapped = ((v - band_low) / range * 255.0).clamp(0.0, 255.0);
        pixel.0[0] = remapped as u8;
    }
}

/// Iterative k-means to find two cluster centroids from a grayscale histogram.
/// Returns (dark_centroid, light_centroid) as f32 values in 0.0..=255.0.
fn two_color_centroids(histogram: &[u32; 256]) -> (f32, f32) {
    // Initial seeds: weighted 25th and 75th percentile
    let total_pixels: u64 = histogram.iter().map(|&c| c as u64).sum();
    if total_pixels == 0 {
        return (0.0, 255.0);
    }

    let mut c0: f32 = percentile_from_histogram(histogram, total_pixels, 0.25);
    let mut c1: f32 = percentile_from_histogram(histogram, total_pixels, 0.75);

    // Iterate k-means until convergence
    for _ in 0..20 {
        let boundary = (c0 + c1) / 2.0;

        let mut sum0: f64 = 0.0;
        let mut count0: f64 = 0.0;
        let mut sum1: f64 = 0.0;
        let mut count1: f64 = 0.0;

        for (i, &freq) in histogram.iter().enumerate() {
            if freq == 0 {
                continue;
            }
            let val = i as f64;
            let f = freq as f64;
            if val <= boundary as f64 {
                sum0 += val * f;
                count0 += f;
            } else {
                sum1 += val * f;
                count1 += f;
            }
        }

        let new_c0 = if count0 > 0.0 {
            (sum0 / count0) as f32
        } else {
            c0
        };
        let new_c1 = if count1 > 0.0 {
            (sum1 / count1) as f32
        } else {
            c1
        };

        // Check convergence
        if (new_c0 - c0).abs() < 0.5 && (new_c1 - c1).abs() < 0.5 {
            break;
        }
        c0 = new_c0;
        c1 = new_c1;
    }

    (c0, c1)
}

/// Find the value at a given percentile from a histogram.
fn percentile_from_histogram(histogram: &[u32; 256], total: u64, percentile: f32) -> f32 {
    let target = (total as f64 * percentile as f64) as u64;
    let mut cumulative: u64 = 0;
    for (i, &freq) in histogram.iter().enumerate() {
        cumulative += freq as u64;
        if cumulative >= target {
            return i as f32;
        }
    }
    255.0
}

pub fn apply_blue_noise_scatter(image: &mut GrayImage, config: &MonochromeConfig) {
    let width = image.width();
    let height = image.height();

    // Pad by 2 to safely handle x-1 and x+1 boundary math
    let mut errors = vec![0.0f32; (width + 2) as usize * height as usize];

    for y in 0..height {
        for x in 0..width {
            let idx = (y * (width + 2) + x + 1) as usize;

            // Fetch the pre-stretched pixel
            let original_luma = image.get_pixel(x, y).0[0] as f32;
            let current_val = original_luma + errors[idx];

            // Fetch blue noise and normalize
            let noise_u8 = get_noise(x, y);
            let noise = (noise_u8 as f32 / 255.0) - 0.5;

            // Jitter Attenuation (Noise Roll-off)
            let distance_from_center = (original_luma - 127.5).abs();
            let attenuation = 1.0 - (distance_from_center / 127.5).clamp(0.0, 1.0);

            // Modulate the threshold
            let dynamic_threshold =
                127.5 + (noise * config.dither_noise * attenuation);

            // Thresholding
            let (new_val, color) = if current_val > dynamic_threshold {
                (255.0, Luma([255]))
            } else {
                (0.0, Luma([0]))
            };
            image.put_pixel(x, y, color);

            // Calculate raw error
            let raw_error = current_val - new_val;

            // Tight Sierra Lite distribution
            if x < width - 1 {
                errors[idx + 1] += raw_error * 0.5; // Right
            }
            if y < height - 1 {
                errors[idx + (width + 2) as usize - 1] += raw_error * 0.25; // Bottom-Left
                errors[idx + (width + 2) as usize] += raw_error * 0.25; // Bottom
            }
        }
    }
}

// Generate blue noise data with the following sequence of commands:
// * Clone or checkout the following commit:
//   https://github.com/mblode/blue-noise-rust/commit/aea756b5853828ac6401937ee39bea27b2f39898
// * Modify the src/generator.rs file to output raw bytes instead of PNG, e.g. by editing the
//   `save_blue_noise_to_png` to contain the following code:
//
//  ```
//  let raw = img.into_raw();
//  BufWriter::new(File::create(&filename).expect("do so"))
//    .write_all(&raw)
//    .expect("write failed");
//  ```
//  * Run `cargo build --release`
//  * Generate the blue noise file with:
//    `./target/release/blue-noise generate --size 64 --output blue-noise.bin`
//  * Copy the binary next to this source file.
const NOISE_DATA: &[u8] = include_bytes!("blue-noise-256.bin");
const NOISE_DATA_WIDTH_AND_HEIGHT: usize = 256;

/// Get the noise value at the given coordinates. If the coordinates are out of bounds,
/// they will wrap around. Means we don't need a noise texture as large as the image.
#[inline]
fn get_noise(x: u32, y: u32) -> u8 {
    let wrap_x = (x as usize) % NOISE_DATA_WIDTH_AND_HEIGHT;
    let wrap_y = (y as usize) % NOISE_DATA_WIDTH_AND_HEIGHT;
    NOISE_DATA[wrap_y * NOISE_DATA_WIDTH_AND_HEIGHT + wrap_x]
}
