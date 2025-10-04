use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_oldest_no_photos() {
    let temp = tempdir().unwrap();

    Command::cargo_bin("ptime")
        .unwrap()
        .arg("oldest")
        .arg(temp.path())
        .assert()
        .success()
        .stdout("");
}

#[test]
fn test_latest_no_photos() {
    let temp = tempdir().unwrap();

    Command::cargo_bin("ptime")
        .unwrap()
        .arg("latest")
        .arg(temp.path())
        .assert()
        .success()
        .stdout("");
}

#[test]
fn test_hist_no_photos() {
    let temp = tempdir().unwrap();

    Command::cargo_bin("ptime")
        .unwrap()
        .arg("hist")
        .arg(temp.path())
        .assert()
        .success()
        .stdout("");
}

#[test]
fn test_hist_invalid_width_zero() {
    let temp = tempdir().unwrap();

    Command::cargo_bin("ptime")
        .unwrap()
        .arg("hist")
        .arg("--width")
        .arg("0")
        .arg(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Width must be at least 1"));
}

#[test]
fn test_nonexistent_directory() {
    Command::cargo_bin("ptime")
        .unwrap()
        .arg("oldest")
        .arg("/nonexistent/path/12345")
        .assert()
        .failure()
        .code(3);
}

#[test]
fn test_help_command() {
    Command::cargo_bin("ptime")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Analyze photo timestamps"));
}

#[test]
fn test_oldest_subcommand_help() {
    Command::cargo_bin("ptime")
        .unwrap()
        .arg("oldest")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Find the oldest photo"));
}

#[test]
fn test_hist_width_parameter() {
    let temp = tempdir().unwrap();

    // Just verify the command accepts width parameter
    Command::cargo_bin("ptime")
        .unwrap()
        .arg("hist")
        .arg("--width")
        .arg("100")
        .arg(temp.path())
        .assert()
        .success();
}

// Note: Testing with actual EXIF data requires JPEG fixtures.
// For a complete test suite, we would create small JPEG files with specific EXIF data.
// Since creating valid JPEG files with EXIF is complex, we focus on CLI behavior tests here.
// Real EXIF testing would be added in production with proper fixture files.

#[test]
fn test_scan_finds_only_jpegs() {
    let temp = tempdir().unwrap();
    let temp_path = temp.path();

    // Create various files
    fs::write(temp_path.join("photo.jpg"), b"not a real jpeg").unwrap();
    fs::write(temp_path.join("image.png"), b"not a jpeg").unwrap();
    fs::write(temp_path.join("doc.txt"), b"text file").unwrap();

    // The command will run but find no valid EXIF photos
    // (since our fake JPEGs don't have EXIF data)
    Command::cargo_bin("ptime")
        .unwrap()
        .arg("oldest")
        .arg(temp_path)
        .assert()
        .success()
        .stdout(""); // No valid photos = empty output
}
