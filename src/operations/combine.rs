use image::{DynamicImage, GenericImageView as _};

use crate::{error::MagickError, image::Image, wm_err};

pub fn combine(
    mut images: Vec<Image>,
    color: image::ColorType,
    fallback: bool,
) -> Result<Image, MagickError> {
    enum ChannelType {
        U8,
        U16,
        U32,
    }

    #[derive(Copy, Clone)]
    enum ChannelCount {
        One,
        Two,
        Three,
        Four,
    }

    fn insert_channel_with_count<Ch: Copy>(
        ch_count: ChannelCount,
        idx: usize,
        matrix: &mut [Ch],
        stride: usize,
        data: &[Ch],
        width: usize,
    ) {
        match ch_count {
            ChannelCount::One => insert_channel::<Ch, 1>(idx, matrix, stride, data, width),
            ChannelCount::Two => insert_channel::<Ch, 2>(idx, matrix, stride, data, width),
            ChannelCount::Three => insert_channel::<Ch, 3>(idx, matrix, stride, data, width),
            ChannelCount::Four => insert_channel::<Ch, 4>(idx, matrix, stride, data, width),
        }
    }

    let color_type = if usize::from(color.channel_count()) < images.len() {
        if !fallback {
            return Err(wm_err!(
                "not enough channels in colorspace {:?} to combine {} images",
                color,
                images.len()
            ));
        }

        // Fallback to true color
        if images.len() <= 3 {
            image::ColorType::Rgb8
        } else {
            image::ColorType::Rgba8
        }
    } else {
        color
    };

    let (ch_type, ch_count) = match color_type {
        image::ColorType::L8 => (ChannelType::U8, ChannelCount::One),
        image::ColorType::La8 => (ChannelType::U8, ChannelCount::Two),
        image::ColorType::Rgb8 => (ChannelType::U8, ChannelCount::Three),
        image::ColorType::Rgba8 => (ChannelType::U8, ChannelCount::Four),
        image::ColorType::L16 => (ChannelType::U16, ChannelCount::One),
        image::ColorType::La16 => (ChannelType::U16, ChannelCount::Two),
        image::ColorType::Rgb16 => (ChannelType::U16, ChannelCount::Three),
        image::ColorType::Rgba16 => (ChannelType::U16, ChannelCount::Four),
        image::ColorType::Rgb32F => (ChannelType::U32, ChannelCount::Three),
        image::ColorType::Rgba32F => (ChannelType::U32, ChannelCount::Four),
        _ => {
            return Err(wm_err!(
                "unsupported color type {:?} for operation combine",
                color
            ))
        }
    };

    let Some(first) = images.first_mut() else {
        return Err(wm_err!("no images found for operation combine"));
    };

    let (width, height) = first.pixels.dimensions();
    let pixels = image::DynamicImage::new(width, height, color_type);

    let mut image = Image {
        format: first.format,
        exif: core::mem::take(&mut first.exif),
        icc: core::mem::take(&mut first.icc),
        pixels,
        properties: first.properties.clone(),
    };

    // We store this as bytes to avoid monomorphizing the channel iteration loop itself over the
    // channel types that we support, which would be unnecessary code bloat.
    let pixel_bytes = as_mut_bytes(&mut image.pixels);

    for (idx, img) in images.iter().enumerate() {
        // The first image dictates the size of the output image.
        let (insert_w, _h) = img.pixels.dimensions();
        let (im_l8, im_l16, im_l32f);

        match ch_type {
            ChannelType::U8 => {
                im_l8 = img.pixels.to_luma8();
                insert_channel_with_count::<u8>(
                    ch_count,
                    idx,
                    bytemuck::cast_slice_mut(pixel_bytes),
                    width as usize,
                    im_l8.as_raw(),
                    insert_w as usize,
                );
            }
            ChannelType::U16 => {
                im_l16 = img.pixels.to_luma16();
                insert_channel_with_count::<u16>(
                    ch_count,
                    idx,
                    bytemuck::cast_slice_mut(pixel_bytes),
                    width as usize,
                    im_l16.as_raw(),
                    insert_w as usize,
                );
            }
            ChannelType::U32 => {
                im_l32f = img.pixels.to_luma32f();
                insert_channel_with_count::<f32>(
                    ch_count,
                    idx,
                    bytemuck::cast_slice_mut(pixel_bytes),
                    width as usize,
                    im_l32f.as_raw(),
                    insert_w as usize,
                );
            }
        };
    }

    Ok(image)
}

// TODO: this method does not exist in `image` but it probably should for `DynamicImage`.
fn as_mut_bytes(img: &mut DynamicImage) -> &mut [u8] {
    let len = usize::from(img.color().bytes_per_pixel())
        * (img.width() as usize)
        * (img.height() as usize);

    let data = match img {
        DynamicImage::ImageLuma8(im) => bytemuck::cast_slice_mut(im.get_mut(..).unwrap()),
        DynamicImage::ImageLumaA8(im) => bytemuck::cast_slice_mut(im.get_mut(..).unwrap()),
        DynamicImage::ImageRgb8(im) => bytemuck::cast_slice_mut(im.get_mut(..).unwrap()),
        DynamicImage::ImageRgba8(im) => bytemuck::cast_slice_mut(im.get_mut(..).unwrap()),
        DynamicImage::ImageLuma16(im) => bytemuck::cast_slice_mut(im.get_mut(..).unwrap()),
        DynamicImage::ImageLumaA16(im) => bytemuck::cast_slice_mut(im.get_mut(..).unwrap()),
        DynamicImage::ImageRgb16(im) => bytemuck::cast_slice_mut(im.get_mut(..).unwrap()),
        DynamicImage::ImageRgba16(im) => bytemuck::cast_slice_mut(im.get_mut(..).unwrap()),
        DynamicImage::ImageRgb32F(im) => bytemuck::cast_slice_mut(im.get_mut(..).unwrap()),
        DynamicImage::ImageRgba32F(im) => bytemuck::cast_slice_mut(im.get_mut(..).unwrap()),
        _ => unreachable!("unsupported color type for combine operation"),
    };

    &mut data[..len]
}

fn insert_channel<Ch: Copy, const CHANNELS: usize>(
    idx: usize,
    matrix: &mut [Ch],
    stride: usize,
    data: &[Ch],
    width: usize,
) {
    assert!(idx < CHANNELS);
    let (matrix, tail) = matrix[idx..].as_chunks_mut::<CHANNELS>();

    // *not* `chunks_exact_mut` because the last row of pixels may be incomplete from the `idx`
    // index shift.
    let lhs_rows = matrix.chunks_mut(stride);
    let rhs_rows = data.chunks_exact(width);

    // The last pixel (group of CHANNELS samples) is incomplete in `lhs_rows` if `idx > 0`. If it
    // should be assigned to then we need to handle it separately. It is not assigned if we do not
    // have enough rows or if the grayscale width is smaller than the stride.
    let assigns_tail = rhs_rows.len() >= lhs_rows.len() && stride >= width;

    for (lhs_row, rhs_row) in lhs_rows.zip(rhs_rows) {
        for (target, source) in lhs_row.iter_mut().zip(rhs_row) {
            target[0] = *source;
        }
    }

    if assigns_tail && !tail.is_empty() {
        tail[0] = *data.last().unwrap();
    }
}

#[test]
fn verify_insert_channel() {
    let mut matrix = [0u8; 12];
    let data = [10u8, 20, 30, 40];

    insert_channel::<u8, 3>(0, &mut matrix, 2, &data, 2);
    assert_eq!(matrix, [10, 0, 0, 20, 0, 0, 30, 0, 0, 40, 0, 0]);

    insert_channel::<u8, 3>(1, &mut matrix, 2, &data, 2);
    assert_eq!(matrix, [10, 10, 0, 20, 20, 0, 30, 30, 0, 40, 40, 0]);

    insert_channel::<u8, 3>(2, &mut matrix, 2, &data, 2);
    assert_eq!(matrix, [10, 10, 10, 20, 20, 20, 30, 30, 30, 40, 40, 40]);
}

#[test]
fn verify_insert_short_too_wide() {
    let mut matrix = [0u8; 12];
    let data = [10u8, 20, 30, 40];

    insert_channel::<u8, 3>(0, &mut matrix, 2, &data, 4);
    assert_eq!(matrix, [10, 0, 0, 20, 0, 0, 0, 0, 0, 0, 0, 0]);

    insert_channel::<u8, 3>(1, &mut matrix, 2, &data, 4);
    assert_eq!(matrix, [10, 10, 0, 20, 20, 0, 0, 0, 0, 0, 0, 0]);
}
