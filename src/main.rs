use std::error::Error;

use fast_image_resize::Resizer;
use image::DynamicImage;

fn main() -> Result<(), Box<dyn Error>> {
    use image::ImageReader;
    let input = std::env::args().nth(1).unwrap();
    let output = std::env::args().nth(2).unwrap();
    let src_image = ImageReader::open(input)?.with_guessed_format()?.decode()?;

    let dst_width = 800;
    let dst_height = 600;
    let mut dst_image = DynamicImage::new(dst_width, dst_height, src_image.color());

    // Create Resizer instance and resize source image
    // into buffer of destination image
    let mut resizer = Resizer::new();
    resizer.resize(&src_image, &mut dst_image, None).unwrap();

    dst_image.save(output)?;
    Ok(())
}
