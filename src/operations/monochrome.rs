use crate::{error::MagickError, image::Image, wm_err};
use image::{DynamicImage, ImageBuffer, Luma};

pub fn monochrome(image: &mut Image) -> Result<(), MagickError> {
    let mut grayscale = image.pixels.to_luma8();
    let noise_texture = NoiseTexture::load()?;
    apply_dithering(&mut grayscale, &noise_texture);
    image.pixels = DynamicImage::ImageLuma8(grayscale);

    Ok(())
}

const BACKGROUND: Luma<u8> = Luma([255]);
const FOREGROUND: Luma<u8> = Luma([0]);

fn apply_dithering(image: &mut ImageBuffer<Luma<u8>, Vec<u8>>, noise_texture: &NoiseTexture) {
    let width = image.width();
    let height = image.height();

    for y in 0..height {
        for x in 0..width {
            let pixel_luma = image.get_pixel(x, y).0[0];
            let noise_luma = noise_texture.get(x, y);

            let color = if pixel_luma > noise_luma {
                BACKGROUND
            } else {
                FOREGROUND
            };

            image.put_pixel(x, y, color);
        }
    }
}

/// Blue noise texture data
pub struct NoiseTexture {
    data: Vec<u8>,
    width: usize,
    height: usize,
}

impl NoiseTexture {
    /// Load blue noise texture from a file
    pub fn load() -> Result<Self, MagickError> {
        let path_to_crate = env!("CARGO_MANIFEST_DIR");

        let img = image::open(format!("{}/src/operations/blue-noise.png", path_to_crate))
            .map_err(|e| wm_err!("failed to load blue noise texture: {}", e))?;

        let gray = img.to_luma8();
        let (width, height) = gray.dimensions();

        if width == 0 || height == 0 {
            return Err(wm_err!("noise texture has invalid dimensions"));
        }

        Ok(Self {
            data: gray.into_raw(),
            width: width as usize,
            height: height as usize,
        })
    }

    /// Get the noise value at the given coordinates. If the coordinates are out of bounds,
    /// they will wrap around. Means we don't need a noise texture as large as the image.
    #[inline]
    fn get(&self, x: u32, y: u32) -> u8 {
        let wrap_x = (x as usize) % self.width;
        let wrap_y = (y as usize) % self.height;
        self.data[wrap_y * self.width + wrap_x]
    }
}
