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
        if let Some(date) = exif
            .fields()
            .filter(|field| field.tag == *tag)
            .find_map(extract_date_from_field)
        {
            return Ok(Some(date));
        }
    }

    // No valid date found
    Ok(None)
}

fn extract_date_from_field(field: &exif::Field) -> Option<NaiveDate> {
    if let exif::Value::Ascii(ref values) = field.value {
        for raw in values {
            if let Ok(text) = std::str::from_utf8(raw) {
                let trimmed = text.trim_matches('\0').trim();
                if trimmed.is_empty() {
                    continue;
                }
                if let Some(date) = parse_exif_datetime(trimmed) {
                    return Some(date);
                }
            }
        }
    }
    None
}

fn parse_exif_datetime(datetime_str: &str) -> Option<NaiveDate> {
    // EXIF datetime format: "YYYY:MM:DD HH:MM:SS"
    // or sometimes just "YYYY:MM:DD". Some readers normalize to hyphen.
    let parts: Vec<&str> = datetime_str.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }

    let date_part = parts[0];
    let separators = [':', '-'];

    for sep in separators {
        let date_components: Vec<&str> = date_part.split(sep).collect();
        if date_components.len() != 3 {
            continue;
        }

        let year = match date_components[0].parse::<i32>() {
            Ok(v) => v,
            Err(_) => continue,
        };
        let month = match date_components[1].parse::<u32>() {
            Ok(v) => v,
            Err(_) => continue,
        };
        let day = match date_components[2].parse::<u32>() {
            Ok(v) => v,
            Err(_) => continue,
        };

        if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
            return Some(date);
        }
    }

    None
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
            Err(err) => {
                if matches!(err, PtimeError::Io(_)) {
                    return Err(err);
                }
                // EXIF parsing or metadata issues are non-fatal
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
    fn test_parse_exif_datetime_with_hyphen() {
        let date = parse_exif_datetime("2020-01-15 10:11:12").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2020, 1, 15).unwrap());
    }

    #[test]
    fn test_parse_exif_datetime_invalid() {
        assert!(parse_exif_datetime("not a date").is_none());
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
