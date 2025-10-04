use crate::error::PtimeError;
use crate::scanner::scan_candidates;
use chrono::NaiveDate;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub struct PhotoMeta {
    pub rel_path: PathBuf,
    pub date: NaiveDate,
}

pub fn read_capture_date(path: &Path) -> Result<Option<NaiveDate>, PtimeError> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    let exif_reader = exif::Reader::new();
    let exif = exif_reader.read_from_container(&mut reader).map_err(|e| {
        PtimeError::Exif(format!(
            "Failed to read EXIF from {}: {}",
            path.display(),
            e
        ))
    })?;

    // Try fields in order: DateTimeOriginal, CreateDate, ModifyDate
    let field_tags = [
        exif::Tag::DateTimeOriginal,
        exif::Tag::DateTime,
        exif::Tag::DateTimeDigitized,
    ];

    for tag in &field_tags {
        if let Some(field) = exif.get_field(*tag, exif::In::PRIMARY) {
            if let Some(date) = parse_exif_datetime(&field.display_value().to_string()) {
                return Ok(Some(date));
            }
        }
    }

    // No valid date found
    Ok(None)
}

fn parse_exif_datetime(datetime_str: &str) -> Option<NaiveDate> {
    // EXIF datetime format: "YYYY:MM:DD HH:MM:SS"
    // or sometimes just "YYYY:MM:DD"
    let parts: Vec<&str> = datetime_str.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }

    let date_part = parts[0];
    let date_components: Vec<&str> = date_part.split(':').collect();

    if date_components.len() != 3 {
        return None;
    }

    let year = date_components[0].parse::<i32>().ok()?;
    let month = date_components[1].parse::<u32>().ok()?;
    let day = date_components[2].parse::<u32>().ok()?;

    NaiveDate::from_ymd_opt(year, month, day)
}

pub fn collect_photos(root: &Path) -> Result<Vec<PhotoMeta>, PtimeError> {
    let candidates = scan_candidates(root)?;
    let mut photos = Vec::new();

    for found in candidates {
        // Try to read capture date, skip if not found or error
        match read_capture_date(&found.abs_path) {
            Ok(Some(date)) => {
                photos.push(PhotoMeta {
                    rel_path: found.rel_path,
                    date,
                });
            }
            Ok(None) => {
                // No date found, skip silently
            }
            Err(_) => {
                // Error reading EXIF, skip silently
            }
        }
    }

    Ok(photos)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_exif_datetime_full() {
        let date = parse_exif_datetime("2023:12:25 14:30:45").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2023, 12, 25).unwrap());
    }

    #[test]
    fn test_parse_exif_datetime_date_only() {
        let date = parse_exif_datetime("2020:01:15").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2020, 1, 15).unwrap());
    }

    #[test]
    fn test_parse_exif_datetime_invalid() {
        assert!(parse_exif_datetime("not a date").is_none());
        assert!(parse_exif_datetime("2023-12-25").is_none()); // wrong separator
        assert!(parse_exif_datetime("2023:13:25 10:00:00").is_none()); // invalid month
        assert!(parse_exif_datetime("").is_none());
    }

    #[test]
    fn test_collect_photos_empty_directory() {
        use tempfile::tempdir;
        let temp = tempdir().unwrap();
        let result = collect_photos(temp.path()).unwrap();
        assert!(result.is_empty());
    }

    // Note: Testing with real EXIF data requires actual JPEG fixtures.
    // For now, we test the parsing logic and empty directory handling.
    // Integration tests with fixtures will be added in Prompt 7.
}
