//! Image rotation operations.

use image::{DynamicImage, ImageBuffer, Pixel, Primitive};
use imageproc::{definitions::Clamp, geometric_transformations::Interpolation};

use crate::{arg_parsers::RotateGeometry, error::MagickError, image::Image};

fn white_pixel<P>() -> P
where
    P: Pixel + Send + Sync,
    P::Subpixel: Send + Sync + Primitive,
{
    let max_value = P::Subpixel::DEFAULT_MAX_VALUE;
    let channels = P::CHANNEL_COUNT as usize;

    // Create a vector with all channels set to maximum value
    let subpixels = vec![max_value; channels];
    P::from_slice(&subpixels).clone()
}

fn rotate_inner<P>(
    image: &mut ImageBuffer<P, Vec<P::Subpixel>>,
    theta: f32,
) -> Result<(), MagickError>
where
    P: 'static + Pixel + Send + Sync,
    P::Subpixel: Send + Sync + Primitive + Into<f32> + Clamp<f32>,
{
    let width = image.width();
    let height = image.height();

    let (sin_theta, cos_theta) = theta.sin_cos();

    let dst_width = (width as f32 * cos_theta + height as f32 * sin_theta).ceil() as u32;
    let dst_height = (width as f32 * sin_theta + height as f32 * cos_theta).ceil() as u32;

    //let dst_size = pic_scale_safe::ImageSize::new(
    //     as usize,
    //    as usize,
    //);
    println!("destination size: {dst_width}x{dst_height}");

    let canvas_width = dst_width.max(width);
    let canvas_height = dst_height.max(height);

    // Create a larger canvas.
    let mut canvas = ImageBuffer::from_pixel(canvas_width, canvas_height, white_pixel::<P>());

    image::imageops::overlay(
        &mut canvas,
        image,
        (canvas_width as i64 - width as i64) / 2,
        (canvas_height as i64 - height as i64) / 2,
    );
    *image = imageproc::geometric_transformations::rotate_about_center(
        &canvas,
        theta,
        Interpolation::Nearest,
        // FIXME: Should account for `-background`.
        white_pixel::<P>(),
    );

    let dst_width = dst_width.max(1);
    let dst_height = dst_height.max(1);

    *image = image::imageops::crop_imm(
        image,
        (canvas_width - dst_width) / 2,
        (canvas_height - dst_height) / 2,
        dst_width,
        dst_height,
    )
    .to_image();

    Ok(())
}

pub fn rotate(image: &mut Image, geometry: &RotateGeometry) -> Result<(), MagickError> {
    let width = image.pixels.width();
    let height = image.pixels.height();
    let theta = geometry.degrees.to_radians() as f32;

    let src_size = pic_scale_safe::ImageSize::new(width as usize, height as usize);
    println!("source size: {:?}", src_size);

    // Check rotation conditions.
    if geometry.only_if_wider && width <= height {
        return Ok(()); // Skip rotation.
    }
    if geometry.only_if_taller && width >= height {
        return Ok(()); // Skip rotation.
    }

    match &mut image.pixels {
        DynamicImage::ImageLuma8(buffer) => {
            rotate_inner(buffer, theta)?;
        }
        DynamicImage::ImageLumaA8(buffer) => {
            rotate_inner(buffer, theta)?;
        }
        DynamicImage::ImageRgb8(buffer) => {
            rotate_inner(buffer, theta)?;
        }
        DynamicImage::ImageRgba8(buffer) => {
            rotate_inner(buffer, theta)?;
        }
        DynamicImage::ImageLuma16(buffer) => {
            rotate_inner(buffer, theta)?;
        }
        DynamicImage::ImageLumaA16(buffer) => {
            rotate_inner(buffer, theta)?;
        }
        DynamicImage::ImageRgb16(buffer) => {
            rotate_inner(buffer, theta)?;
        }
        DynamicImage::ImageRgba16(buffer) => {
            rotate_inner(buffer, theta)?;
        }
        DynamicImage::ImageRgb32F(buffer) => {
            rotate_inner(buffer, theta)?;
        }
        DynamicImage::ImageRgba32F(buffer) => {
            rotate_inner(buffer, theta)?;
        }
        _ => unreachable!(),
    }

    Ok(())
}
