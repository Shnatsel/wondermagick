use std::fs;
use std::process::Command;

#[test]
fn test_convert_png_to_jpg_succeeds() {
    let binary = env!("CARGO_BIN_EXE_wm-convert");
    let tmp_dir = env!("CARGO_TARGET_TMPDIR");
    let output_path = format!("{}/resized.jpg", tmp_dir);
    let _ = fs::remove_file(&output_path);

    let result = Command::new(binary)
        .args(&["./tests/sample.png", &output_path])
        .output()
        .expect("convert did not exit successfully");

    assert!(result.status.success());
    assert!(std::path::Path::new(&output_path).exists());
}

#[test]
fn test_resize_identify_succeeds() {
    let binary = env!("CARGO_BIN_EXE_wm-convert");
    let tmp_dir = env!("CARGO_TARGET_TMPDIR");
    let output_path = format!("{}/resized.png", tmp_dir);
    let _ = fs::remove_file(&output_path);

    let convert = Command::new(binary)
        .args(&["./tests/sample.png", "-resize", "5x5", &output_path])
        .output()
        .expect("convert did not exit successfully");

    assert!(convert.status.success());
    assert!(std::path::Path::new(&output_path).exists());

    let identify = Command::new(binary)
        .args(&[&output_path, "-identify", &output_path])
        .output()
        .expect("convert did not exit successfully");

    assert!(convert.status.success());
    assert!(String::from_utf8(identify.stdout).unwrap().contains("5x5"));
}

#[test]
fn test_write_as_intermediate() {
    let binary = env!("CARGO_BIN_EXE_wm-convert");
    let tmp_dir = env!("CARGO_TARGET_TMPDIR");

    let output_path = format!("{}/resized.jpg", tmp_dir);
    let copy_path = format!("{}/copy.png", tmp_dir);
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&copy_path);

    let result = Command::new(binary)
        .args(&[
            "./tests/sample.png",
            "-write",
            &copy_path,
            "-resize",
            "50%x50%",
            &output_path,
        ])
        .output()
        .expect("convert did not exit successfully");

    assert!(result.status.success());
    assert!(std::path::Path::new(&output_path).exists());
    assert!(std::path::Path::new(&copy_path).exists());

    let copy_img = image::open(&copy_path).unwrap();
    let output_img = image::open(&output_path).unwrap();

    use image::GenericImageView as _;
    assert_ne!(copy_img.dimensions(), output_img.dimensions());
}
