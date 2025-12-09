use std::fs;
use std::process::Command;

// The individual tests may be expected in parallel but we are not prepared for ensuring they do
// not accidentally attempt to write the same files (and then test the files with contradictory
// expectations, finding the content the other test has written).
static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
fn test_convert_png_to_jpg_succeeds() {
    let _guard = LOCK.lock().unwrap();

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
    let _guard = LOCK.lock().unwrap();

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
fn combine_succeeds() {
    let _guard = LOCK.lock().unwrap();

    let binary = env!("CARGO_BIN_EXE_wm-convert");
    let tmp_dir = env!("CARGO_TARGET_TMPDIR");
    let output_path = format!("{}/stacked.png", tmp_dir);
    let _ = fs::remove_file(&output_path);

    let convert = Command::new(binary)
        .args(&[
            "./tests/sample.png",
            // Make sure this can not be compressed to grayscale in the output write.
            "-negate",
            "./tests/sample.png",
            "./tests/sample.png",
            "-combine",
            &output_path,
        ])
        .output()
        .expect("convert did not exit successfully");

    assert!(convert.status.success());
    assert!(std::path::Path::new(&output_path).exists());

    let identify = Command::new(binary)
        .args(&[&output_path, "-identify", &output_path])
        .output()
        .expect("convert did not exit successfully");

    assert!(identify.status.success());

    assert!(String::from_utf8(identify.stdout).unwrap().contains("sRGB"));
}

#[test]
fn test_write_as_intermediate() {
    let _guard = LOCK.lock().unwrap();

    let binary = env!("CARGO_BIN_EXE_wm-convert");
    let tmp_dir = env!("CARGO_TARGET_TMPDIR");

    let output_path = format!("{}/resized.jpg", tmp_dir);
    let copy_path = format!("{}/copy.png", tmp_dir);
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&copy_path);

    let write = Command::new(binary)
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

    assert!(write.status.success());
    assert!(std::path::Path::new(&output_path).exists());
    assert!(std::path::Path::new(&copy_path).exists());

    let copy_img = image::open(&copy_path).unwrap();
    let output_img = image::open(&output_path).unwrap();

    use image::GenericImageView as _;
    assert_ne!(
        copy_img.dimensions(),
        output_img.dimensions(),
        "Intermediate write should have twice the dimensions of the final output",
    );
}

#[test]
fn combine_upgrades_to_rgba() {
    let _guard = LOCK.lock().unwrap();

    let binary = env!("CARGO_BIN_EXE_wm-convert");
    let tmp_dir = env!("CARGO_TARGET_TMPDIR");
    let output_path = format!("{}/stacked.png", tmp_dir);
    let _ = fs::remove_file(&output_path);

    let convert = Command::new(binary)
        .args(&[
            "./tests/sample.png",
            // Make sure this can not be compressed to grayscale in the output write.
            "-negate",
            "./tests/sample.png",
            "./tests/sample.png",
            "./tests/sample.png",
            "-combine",
            &output_path,
        ])
        .output()
        .expect("convert did not exit successfully");

    assert!(convert.status.success());
    assert!(std::path::Path::new(&output_path).exists());

    let identify = Command::new(binary)
        .args(&[&output_path, "-identify", &output_path])
        .output()
        .expect("convert did not exit successfully");

    assert!(identify.status.success());

    assert!(String::from_utf8(identify.stdout).unwrap().contains("sRGB"));
}

#[test]
fn combine_as_gray() {
    let _guard = LOCK.lock().unwrap();

    let binary = env!("CARGO_BIN_EXE_wm-convert");
    let tmp_dir = env!("CARGO_TARGET_TMPDIR");
    let output_path = format!("{}/stacked.png", tmp_dir);
    let _ = fs::remove_file(&output_path);

    let convert = Command::new(binary)
        .args(&[
            "./tests/sample.png",
            "-colorspace",
            "gray",
            "-combine",
            &output_path,
        ])
        .output()
        .expect("convert did not exit successfully");

    assert!(convert.status.success());
    assert!(std::path::Path::new(&output_path).exists());

    let identify = Command::new(binary)
        .args(&[&output_path, "-identify", &output_path])
        .output()
        .expect("convert did not exit successfully");

    assert!(identify.status.success());

    assert!(String::from_utf8(identify.stdout).unwrap().contains("Gray"));
}

#[test]
fn combine_as_gray_upgrades() {
    let _guard = LOCK.lock().unwrap();

    let binary = env!("CARGO_BIN_EXE_wm-convert");
    let tmp_dir = env!("CARGO_TARGET_TMPDIR");
    let output_path = format!("{}/stacked.png", tmp_dir);
    let _ = fs::remove_file(&output_path);

    let convert = Command::new(binary)
        .args(&[
            "./tests/sample.png",
            "-colorspace",
            "gray",
            "-negate",
            "./tests/sample.png",
            "./tests/sample.png",
            "-combine",
            &output_path,
        ])
        .output()
        .expect("convert did not exit successfully");

    assert!(convert.status.success());
    assert!(std::path::Path::new(&output_path).exists());

    let identify = Command::new(binary)
        .args(&[&output_path, "-identify", &output_path])
        .output()
        .expect("convert did not exit successfully");

    assert!(identify.status.success());

    // Even though we requested colorspace, by exceeding the channel count we forced combined to
    // upgrade us to an sRGB color model. Had we four we'd also get an alpha channel.
    assert!(String::from_utf8(identify.stdout).unwrap().contains("sRGB"));
}

#[test]
fn combine_plus_works() {
    let _guard = LOCK.lock().unwrap();

    let binary = env!("CARGO_BIN_EXE_wm-convert");
    let tmp_dir = env!("CARGO_TARGET_TMPDIR");
    let output_path = format!("{}/stacked.png", tmp_dir);
    let _ = fs::remove_file(&output_path);

    let convert = Command::new(binary)
        .args(&[
            "./tests/sample.png",
            "-negate",
            "./tests/sample.png",
            "./tests/sample.png",
            "+combine",
            "srgb",
            &output_path,
        ])
        .output()
        .expect("convert did not exit successfully");

    assert!(convert.status.success());
    assert!(std::path::Path::new(&output_path).exists());

    let identify = Command::new(binary)
        .args(&[&output_path, "-identify", &output_path])
        .output()
        .expect("convert did not exit successfully");

    assert!(identify.status.success());

    // Even though we requested colorspace, by exceeding the channel count we forced combined to
    // upgrade us to an sRGB color model. Had we four we'd also get an alpha channel.
    assert!(String::from_utf8(identify.stdout).unwrap().contains("sRGB"));
}

// Note: this diverges from imagemagick. The documentation says that `+combine <colorspace>`
// precisely controls the output channel list but it does not detail how additional input images
// are actually merged into the output image when doing so. And the observable changes are just
// confusing and vaguely looks like corruption. See `Args::Combine` in `src/plan.rs`.
#[test]
fn combine_plus_is_precise() {
    let _guard = LOCK.lock().unwrap();

    let binary = env!("CARGO_BIN_EXE_wm-convert");
    let tmp_dir = env!("CARGO_TARGET_TMPDIR");
    let output_path = format!("{}/stacked.png", tmp_dir);
    let _ = fs::remove_file(&output_path);

    let convert = Command::new(binary)
        .args(&[
            "./tests/sample.png",
            "./tests/sample.png",
            "+combine",
            "gray",
            &output_path,
        ])
        .output()
        .expect("convert did not exit successfully");

    assert!(!convert.status.success());
}
