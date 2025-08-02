use std::path::{Path, PathBuf};

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
    let wondermagick_output_path = directory.path().join("wondermagick_output.png");
    let imagemagick_output_path = directory.path().join("imagemagick_output.png");

    let wm_convert_status = std::process::Command::new("target/debug/wm-convert")
        .args(extra_arguments)
        .arg(&wondermagick_output_path)
        .status()
        .expect("must have succeeded");

    let magick_status = std::process::Command::new("magick")
        .arg("convert")
        .args(extra_arguments)
        .arg(&imagemagick_output_path)
        .status()
        .expect("must have succeeded");

    if !wm_convert_status.success() {
        panic!("wondermagick command failed");
    }
    if !magick_status.success() {
        panic!("imagemagick command failed");
    }

    let dssim_score = compute_visual_diff(&wondermagick_output_path, &imagemagick_output_path);
    if dssim_score > DSSIM_TOLERANCE {
        panic!("High DSSIM score {dssim_score} for arguments: {extra_arguments:?}");
    }

    (wondermagick_output_path, imagemagick_output_path)
}
