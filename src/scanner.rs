use crate::error::PtimeError;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, PartialEq)]
pub struct FoundFile {
    pub rel_path: PathBuf,
    pub abs_path: PathBuf,
}

pub fn scan_candidates(root: &Path) -> Result<Vec<FoundFile>, PtimeError> {
    // Canonicalize the root to get absolute path
    let canonical_root = root
        .canonicalize()
        .map_err(|e| PtimeError::CanonicalizationError {
            path: root.to_path_buf(),
            source: e,
        })?;

    let mut results = Vec::new();

    for entry in WalkDir::new(&canonical_root)
        .follow_links(false)
        .into_iter()
    {
        let entry = entry.map_err(|e| {
            let path = e.path().unwrap_or(root).to_path_buf();
            PtimeError::DirectoryReadError {
                path,
                source: e.into(),
            }
        })?;

        // Skip directories
        if !entry.file_type().is_file() {
            continue;
        }

        let abs_path = entry.path();

        // Check if it's a JPEG file
        if !is_jpeg_extension(abs_path) {
            continue;
        }

        // Compute relative path
        let rel_path = compute_relative_path(&canonical_root, abs_path)?;

        results.push(FoundFile {
            rel_path,
            abs_path: abs_path.to_path_buf(),
        });
    }

    Ok(results)
}

fn is_jpeg_extension(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext_lower = ext.to_string_lossy().to_lowercase();
        ext_lower == "jpg" || ext_lower == "jpeg"
    } else {
        false
    }
}

fn compute_relative_path(root: &Path, abs_path: &Path) -> Result<PathBuf, PtimeError> {
    abs_path
        .strip_prefix(root)
        .map(|p| p.to_path_buf())
        .map_err(|_| PtimeError::RelativePathError {
            path: abs_path.to_path_buf(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_scan_empty_directory() {
        let temp = tempdir().unwrap();
        let result = scan_candidates(temp.path()).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_scan_finds_jpeg_files() {
        let temp = tempdir().unwrap();
        let temp_path = temp.path();

        // Create test files
        fs::write(temp_path.join("photo1.jpg"), b"fake jpeg").unwrap();
        fs::write(temp_path.join("photo2.JPEG"), b"fake jpeg").unwrap();
        fs::write(temp_path.join("photo3.JPG"), b"fake jpeg").unwrap();
        fs::write(temp_path.join("document.txt"), b"not a jpeg").unwrap();
        fs::write(temp_path.join("image.png"), b"not a jpeg").unwrap();

        let result = scan_candidates(temp_path).unwrap();
        assert_eq!(result.len(), 3);

        let rel_paths: Vec<_> = result.iter().map(|f| f.rel_path.clone()).collect();
        assert!(rel_paths.contains(&PathBuf::from("photo1.jpg")));
        assert!(rel_paths.contains(&PathBuf::from("photo2.JPEG")));
        assert!(rel_paths.contains(&PathBuf::from("photo3.JPG")));
    }

    #[test]
    fn test_scan_nested_directories() {
        let temp = tempdir().unwrap();
        let temp_path = temp.path();

        // Create nested structure
        fs::create_dir(temp_path.join("subdir")).unwrap();
        fs::create_dir(temp_path.join("subdir/nested")).unwrap();

        fs::write(temp_path.join("root.jpg"), b"fake").unwrap();
        fs::write(temp_path.join("subdir/photo.jpg"), b"fake").unwrap();
        fs::write(temp_path.join("subdir/nested/deep.jpeg"), b"fake").unwrap();

        let result = scan_candidates(temp_path).unwrap();
        assert_eq!(result.len(), 3);

        let rel_paths: Vec<_> = result.iter().map(|f| f.rel_path.clone()).collect();
        assert!(rel_paths.contains(&PathBuf::from("root.jpg")));
        assert!(rel_paths.contains(&PathBuf::from("subdir/photo.jpg")));
        assert!(rel_paths.contains(&PathBuf::from("subdir/nested/deep.jpeg")));
    }

    #[test]
    fn test_scan_current_directory_canonicalization() {
        // Test that "." gets canonicalized properly
        let temp = tempdir().unwrap();
        let temp_path = temp.path();
        fs::write(temp_path.join("test.jpg"), b"fake").unwrap();

        // Change to temp directory and scan "."
        let old_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_path).unwrap();

        let result = scan_candidates(Path::new(".")).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].rel_path, PathBuf::from("test.jpg"));

        // Restore original directory
        std::env::set_current_dir(old_dir).unwrap();
    }

    #[test]
    fn test_is_jpeg_extension() {
        assert!(is_jpeg_extension(Path::new("photo.jpg")));
        assert!(is_jpeg_extension(Path::new("photo.jpeg")));
        assert!(is_jpeg_extension(Path::new("photo.JPG")));
        assert!(is_jpeg_extension(Path::new("photo.JPEG")));
        assert!(is_jpeg_extension(Path::new("photo.JpG")));

        assert!(!is_jpeg_extension(Path::new("photo.png")));
        assert!(!is_jpeg_extension(Path::new("photo.gif")));
        assert!(!is_jpeg_extension(Path::new("photo")));
        assert!(!is_jpeg_extension(Path::new("photo.txt")));
    }

    #[test]
    fn test_compute_relative_path() {
        let root = Path::new("/base/path");
        let abs_path = Path::new("/base/path/subdir/file.jpg");

        let rel = compute_relative_path(root, abs_path).unwrap();
        assert_eq!(rel, PathBuf::from("subdir/file.jpg"));
    }
}
