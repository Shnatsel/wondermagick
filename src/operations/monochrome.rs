use crate::{error::MagickError, image::Image};
use image::{DynamicImage, GrayImage, Luma};

// Diffusion dampening for error that pushes dark pixels toward white.
// At 1.0 full error propagates (ImageMagick 6 Floyd-Steinberg behavior).
// Lower values suppress dithering in shadows, producing solid black areas.
const SHADOW_DIFFUSION: f32 = 1.0;

// Diffusion dampening for error that pushes bright pixels toward black.
// At 1.0 full error propagates (ImageMagick 6 Floyd-Steinberg behavior).
// Lower values suppress dithering in highlights, producing solid white areas.
const HIGHLIGHT_DIFFUSION: f32 = 1.0;

//   0.0: Pure Math (No Jitter)
//  20.0 to 60.0: Subtle Breakup
//  80.0 to 140.0: The "Film Grain" Sweet Spot
// 150.0 to 255.0: Heavy Noise
const THRESHOLD_JITTER: f32 = 100.0;

// Controls how far below the dark centroid the dither band extends.
// 0.0 = dither starts exactly at the dark centroid (IM6 default, hard black threshold)
// 0.5 = extends halfway from the dark centroid toward pure black
// 1.0 = extends all the way to pure black (maximum shadow graduation)
const DITHER_BAND_SHADOW_EXPANSION: f32 = 0.0;

// Controls how far above the light centroid the dither band extends.
// 0.0 = dither ends exactly at the light centroid (IM6 default, hard white threshold)
// 0.5 = extends halfway from the light centroid toward pure white
// 1.0 = extends all the way to pure white (maximum highlight graduation)
const DITHER_BAND_HIGHLIGHT_EXPANSION: f32 = 0.0;

pub fn monochrome(image: &mut Image) -> Result<(), MagickError> {
    let mut grayscaled = image.pixels.to_luma8();
    remap_to_dither_band(&mut grayscaled);
    apply_blue_noise_scatter(&mut grayscaled);
    image.pixels = DynamicImage::ImageLuma8(grayscaled);
    image.set_color_type_from_pixels();
    Ok(())
}

/// Two-color quantization matching ImageMagick 6's `-monochrome` behavior.
///
/// IM6 does not dither between pure 0 and 255. Instead it finds two
/// representative gray levels (cluster centroids) from the image histogram,
/// then dithers only between those. Pixels darker than the dark centroid
/// are hard-thresholded to black, pixels brighter than the light centroid
/// are hard-thresholded to white, and only the tones in between get dithered.
///
/// The DITHER_BAND_SHADOW_EXPANSION and DITHER_BAND_HIGHLIGHT_EXPANSION
/// constants let you widen the dither range beyond the centroids.
/// At 0.0 you get the IM6-like hard threshold at the extremes.
/// At 1.0 the dither band reaches all the way to 0 or 255, giving
/// full graduation in shadows or highlights respectively.
fn remap_to_dither_band(image: &mut GrayImage) {
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
    let band_low = c_dark - (c_dark * DITHER_BAND_SHADOW_EXPANSION);
    let band_high = c_light + ((255.0 - c_light) * DITHER_BAND_HIGHLIGHT_EXPANSION);

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

pub fn apply_blue_noise_scatter(image: &mut GrayImage) {
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
            let dynamic_threshold = 127.5 + (noise * THRESHOLD_JITTER * attenuation);

            // Thresholding
            let (new_val, color) = if current_val > dynamic_threshold {
                (255.0, Luma([255]))
            } else {
                (0.0, Luma([0]))
            };
            image.put_pixel(x, y, color);

            // Calculate raw error
            let raw_error = current_val - new_val;

            // Apply asymmetric damping
            let tuned_error = if raw_error > 0.0 {
                raw_error * SHADOW_DIFFUSION
            } else {
                raw_error * HIGHLIGHT_DIFFUSION
            };

            // Tight Sierra Lite distribution
            if x < width - 1 {
                errors[idx + 1] += tuned_error * 0.5; // Right
            }
            if y < height - 1 {
                errors[idx + (width + 2) as usize - 1] += tuned_error * 0.25; // Bottom-Left
                errors[idx + (width + 2) as usize] += tuned_error * 0.25; // Bottom
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
