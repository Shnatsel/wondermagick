#![no_main]

use std::path::Path;

use arbitrary::Unstructured;
use bounded_integer::BoundedU16;
use image::GenericImageView;
use libfuzzer_sys::fuzz_target;

const MAX_WIDTH: u16 = 1024;
const MAX_HEIGHT: u16 = 1024;

#[derive(Debug)]
struct StructuredImage {
    width: BoundedU16<1, MAX_WIDTH>,
    height: BoundedU16<1, MAX_HEIGHT>,
    rgb_data: Vec<u8>,
}

impl StructuredImage {
    fn save_as_png(&self, path: impl AsRef<Path>) -> Result<(), std::io::Error> {
        use image::{codecs::png::PngEncoder, ImageBuffer, ImageEncoder, RgbImage};
        use std::fs::File;

        let img: RgbImage =
            ImageBuffer::from_fn(self.width.get() as u32, self.height.get() as u32, |x, y| {
                let idx = (y * self.width.get() as u32 + x) as usize * 3;
                image::Rgb([
                    self.rgb_data[idx],
                    self.rgb_data[idx + 1],
                    self.rgb_data[idx + 2],
                ])
            });

        let file = File::create(path)?;
        let encoder = PngEncoder::new_with_quality(
            file,
            image::codecs::png::CompressionType::Fast,
            image::codecs::png::FilterType::NoFilter,
        );
        encoder
            .write_image(
                &img,
                self.width.get() as u32,
                self.height.get() as u32,
                image::ExtendedColorType::Rgb8,
            )
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.to_string()))
    }
}

impl<'a> arbitrary::Arbitrary<'a> for StructuredImage {
    fn arbitrary(unstructured: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let width: BoundedU16<1, MAX_WIDTH> = unstructured.arbitrary()?;
        let height: BoundedU16<1, MAX_HEIGHT> = unstructured.arbitrary()?;
        let rgb_data_len = width.get() as usize * height.get() as usize * 3;
        let rgb_data = unstructured.bytes(rgb_data_len)?;

        Ok(Self {
            width,
            height,
            rgb_data: rgb_data.to_vec(),
        })
    }
}

fuzz_target!(|input: (
    StructuredImage,
    BoundedU16<1, MAX_WIDTH>,
    BoundedU16<1, MAX_HEIGHT>
)| {
    let (image, new_width, new_height) = input;
    let new_width = new_width.get() as u32;
    let new_height = new_height.get() as u32;

    let temp_directory = tempfile::tempdir().expect("failed to create temporary directory");
    let input_path = temp_directory.path().join("input_image.png");
    image
        .save_as_png(&input_path)
        .expect("failed to save image as PNG");

    let (wondermagick_output_image_path, imagemagick_output_image_path) =
        wondermagick_fuzz::run_commands_and_compare(
            &temp_directory,
            &[
                input_path.to_str().expect("must be valid"),
                "-resize",
                &format!("{}x{}!", new_width, new_height),
            ],
        );

    let (wondermagick_output_image_width, wondermagick_output_image_height) =
        image::open(&wondermagick_output_image_path)
            .expect("could not open the WonderMagick output file")
            .dimensions();
    let (imagemagick_output_image_width, imagemagick_output_image_height) =
        image::open(&imagemagick_output_image_path)
            .expect("could not open the WonderMagick output file")
            .dimensions();

    // Contract.
    assert_eq!(imagemagick_output_image_width, new_width);
    assert_eq!(imagemagick_output_image_height, new_height);

    // Test assertion.
    assert_eq!(
        wondermagick_output_image_width,
        new_width,
        "{}",
        wondermagick_output_image_path.display()
    );
    assert_eq!(
        wondermagick_output_image_height,
        new_height,
        "{}",
        wondermagick_output_image_path.display()
    );
});
