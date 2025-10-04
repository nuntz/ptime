use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PtimeError {
    #[error("Failed to canonicalize path {path}: {source}")]
    CanonicalizationError {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("Failed to read directory {path}: {source}")]
    DirectoryReadError {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("Failed to compute relative path for {path}")]
    RelativePathError { path: PathBuf },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("EXIF error: {0}")]
    Exif(String),
}

impl PtimeError {
    pub fn exit_code(&self) -> i32 {
        match self {
            PtimeError::Io(_)
            | PtimeError::CanonicalizationError { .. }
            | PtimeError::DirectoryReadError { .. } => 3,
            _ => 1,
        }
    }
}
