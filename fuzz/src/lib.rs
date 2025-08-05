use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

use tempfile::TempDir;

fn compute_visual_diff(wondermagick_path: &Path, imagemagick_path: &Path) -> f64 {
    let wm_image = image::open(wondermagick_path)
        .expect("could not open the file")
        .to_rgb8();
    let magick_image = image::open(imagemagick_path)
        .expect("could not open the file")
        .to_rgb8();

    let config = dssim::Dssim::new();

    let wm_rgb: Vec<rgb::RGB<u8>> = wm_image
        .pixels()
        .map(|p| rgb::RGB::from([p[0], p[1], p[2]]))
        .collect();
    let magick_rgb: Vec<rgb::RGB<u8>> = magick_image
        .pixels()
        .map(|p| rgb::RGB::from([p[0], p[1], p[2]]))
        .collect();

    let wm_dssim = config
        .create_image_rgb(
            &wm_rgb,
            wm_image.width() as usize,
            wm_image.height() as usize,
        )
        .expect("failed to create dssim image from WonderMagick output");
    let magick_dssim = config
        .create_image_rgb(
            &magick_rgb,
            magick_image.width() as usize,
            magick_image.height() as usize,
        )
        .expect("failed to create dssim image from ImageMagick output");

    let (dssim_score, _) = config.compare(&wm_dssim, &magick_dssim);
    dssim_score.into()
}

const DSSIM_TOLERANCE: f64 = 0.05;

pub fn run_commands_and_compare(
    directory: &TempDir,
    extra_arguments: &[&str],
) -> (PathBuf, PathBuf) {
    use image::GenericImageView as _;

    let wondermagick_output_path = directory.path().join("wondermagick_output.png");
    let imagemagick_output_path = directory.path().join("imagemagick_output.png");

    let mut common_arguments = extra_arguments.to_vec();
    common_arguments.push("-quality");
    common_arguments.push("11");

    let plan = {
        let mut wm_arguments = vec![OsString::from("target/release/wm-convert")];
        wm_arguments.extend(common_arguments.iter().map(OsString::from));
        wm_arguments.push(wondermagick_output_path.as_os_str().to_os_string());
        wondermagick::args::parse_args(wm_arguments).expect("must have succeeded")
    };
    plan.execute().expect("must have succeeded");

    let magick_status = std::process::Command::new("convert")
        .args(common_arguments)
        .arg(&imagemagick_output_path)
        .status()
        .expect("must have succeeded");

    if !magick_status.success() {
        panic!("imagemagick command failed");
    }

    let (wondermagick_output_image_width, wondermagick_output_image_height) =
        image::open(&wondermagick_output_path)
            .expect("could not open the WonderMagick output file")
            .dimensions();
    let (imagemagick_output_image_width, imagemagick_output_image_height) =
        image::open(&imagemagick_output_path)
            .expect("could not open the WonderMagick output file")
            .dimensions();

    assert_eq!(
        imagemagick_output_image_width,
        wondermagick_output_image_width
    );
    assert_eq!(
        imagemagick_output_image_height,
        wondermagick_output_image_height
    );

    let dssim_score = compute_visual_diff(&wondermagick_output_path, &imagemagick_output_path);
    if dssim_score > DSSIM_TOLERANCE {
        panic!("High DSSIM score {dssim_score} for arguments: {extra_arguments:?}");
    }

    (wondermagick_output_path, imagemagick_output_path)
}
