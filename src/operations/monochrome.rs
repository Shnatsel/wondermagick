use crate::{error::MagickError, image::Image};
use image::{DynamicImage, GrayImage, Luma};

pub fn monochrome(image: &mut Image) -> Result<(), MagickError> {
    let mut grayscaled = image.pixels.to_luma8();
    apply_contrast(&mut grayscaled, CONTRAST_FACTOR);
    apply_dithering(&mut grayscaled);
    image.pixels = DynamicImage::ImageLuma8(grayscaled);

    Ok(())
}

/// Empirically tuned to give results similar to ImageMagick's -monochrome
const CONTRAST_FACTOR: f32 = 2.0;

fn apply_contrast(image: &mut GrayImage, contrast_factor: f32) {
    let offset = 128.0 * (1.0 - contrast_factor);

    for pixel in image.pixels_mut() {
        for channel in pixel.0.iter_mut() {
            let value = *channel as f32;
            let adjusted = (value * contrast_factor + offset).clamp(0.0, 255.0);
            *channel = adjusted as u8;
        }
    }
}

const WHITE: Luma<u8> = Luma([255]);
const BLACK: Luma<u8> = Luma([0]);

fn apply_dithering(image: &mut GrayImage) {
    let width = image.width();
    let height = image.height();

    for y in 0..height {
        for x in 0..width {
            let pixel_luma = image.get_pixel(x, y).0[0];

            let noise_luma = get_noise(x, y);
            let color = if pixel_luma > noise_luma {
                WHITE
            } else if pixel_luma < noise_luma {
                BLACK
            // tie break: pixel and noise have the same value, select the nearest color
            } else if pixel_luma > 127 {
                WHITE
            } else {
                BLACK
            };

            image.put_pixel(x, y, color);
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
const NOISE_DATA: &[u8] = include_bytes!("blue-noise.bin");
const NOISE_DATA_WIDTH_AND_HEIGHT: usize = 64;

/// Get the noise value at the given coordinates. If the coordinates are out of bounds,
/// they will wrap around. Means we don't need a noise texture as large as the image.
#[inline]
fn get_noise(x: u32, y: u32) -> u8 {
    let wrap_x = (x as usize) % NOISE_DATA_WIDTH_AND_HEIGHT;
    let wrap_y = (y as usize) % NOISE_DATA_WIDTH_AND_HEIGHT;
    NOISE_DATA[wrap_y * NOISE_DATA_WIDTH_AND_HEIGHT + wrap_x]
}
