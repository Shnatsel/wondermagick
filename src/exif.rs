use image::DynamicImage;

pub fn apply_exif_orientation(
    image: &mut DynamicImage,
    orientation: u8,
) -> Result<(), &'static str> {
    // An explanation of Exif orientation:
    // https://web.archive.org/web/20200412005226/https://www.impulseadventure.com/photo/exif-orientation.html
    match orientation {
        1 => Ok(()), // no transformations needed
        2 => Ok(image.fliph_in_place()),
        3 => Ok(image.rotate180_in_place()),
        4 => Ok(image.flipv_in_place()),
        5 => {
            let mut new_image = image.rotate90();
            new_image.fliph_in_place();
            *image = new_image;
            Ok(())
        }
        6 => Ok(*image = image.rotate90()),
        7 => {
            let mut new_image = image.rotate270();
            new_image.fliph_in_place();
            *image = new_image;
            Ok(())
        }
        8 => Ok(*image = image.rotate270()),
        0 | 9.. => return Err("Invalid Exif orientation value"),
    }
}