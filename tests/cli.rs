use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
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
        .code(2)
        .stderr(predicate::str::contains("invalid value"));
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

#[test]
fn test_latest_with_exif_fixture() {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");

    Command::cargo_bin("ptime")
        .unwrap()
        .arg("latest")
        .arg(&fixtures)
        .assert()
        .success()
        .stdout("sample_exif.jpg 2025-06-07\n");
}

#[cfg(unix)]
#[test]
fn test_permission_denied_propagates_io_error() {
    use std::fs::Permissions;
    use std::os::unix::fs::PermissionsExt;

    let temp = tempdir().unwrap();
    let photo_path = temp.path().join("restricted.jpg");
    fs::write(&photo_path, b"fake jpeg").unwrap();
    fs::set_permissions(&photo_path, Permissions::from_mode(0o000)).unwrap();

    Command::cargo_bin("ptime")
        .unwrap()
        .arg("oldest")
        .arg(temp.path())
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("IO error"));

    fs::set_permissions(&photo_path, Permissions::from_mode(0o600)).unwrap();
}
