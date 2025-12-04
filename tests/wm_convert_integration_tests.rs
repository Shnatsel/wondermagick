use std::fs;
use std::process::Command;

fn setup<'a>() -> (&'a str, &'a str) {
    let binary = env!("CARGO_BIN_EXE_wm-convert");
    let tmp_dir = env!("CARGO_TARGET_TMPDIR");
    (binary, tmp_dir)
}

#[test]
fn test_convert_png_to_jpg_succeeds() {
    let (binary, tmp_dir) = setup();
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
fn test_convert_composite_succeeds() {
    let (binary, tmp_dir) = setup();
    let output_path = format!("{}/composited.png", tmp_dir);
    let _ = fs::remove_file(&output_path);

    let result = Command::new(binary)
        .args(&[
            "./tests/sample.png",
            "./tests/sample.png",
            "-composite",
            &output_path,
        ])
        .output()
        .expect("convert did not exit successfully");

    assert!(result.status.success());
    assert!(std::path::Path::new(&output_path).exists());
}

#[test]
fn test_resize_identify_succeeds() {
    let (binary, tmp_dir) = setup();
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
