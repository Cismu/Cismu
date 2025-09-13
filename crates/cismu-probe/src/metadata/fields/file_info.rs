use std::{fs, io, path::PathBuf, time::SystemTime};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FileInfoError {
    #[error("File not found at path: {0:?}")]
    FileNotFound(PathBuf),

    #[error("Path contains invalid UTF-8 characters: {0:?}")]
    InvalidPath(PathBuf),

    #[error("I/O error for path {path:?}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

#[derive(Debug)]
pub struct FileInfo {
    path: PathBuf,
    name: String,
    extension: String,
    file_size: u64,
    modified_date: SystemTime,
}

impl FileInfo {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, FileInfoError> {
        let path = path.into();

        let metadata = fs::metadata(&path).map_err(|e| {
            if e.kind() == io::ErrorKind::NotFound {
                FileInfoError::FileNotFound(path.clone())
            } else {
                FileInfoError::Io {
                    path: path.clone(),
                    source: e,
                }
            }
        })?;

        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| FileInfoError::InvalidPath(path.clone()))?
            .to_string();

        let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_string();

        let modified_date = metadata.modified().map_err(|e| FileInfoError::Io {
            path: path.clone(),
            source: e,
        })?;

        Ok(Self {
            path,
            name,
            extension,
            file_size: metadata.len(),
            modified_date,
        })
    }

    pub fn has_changed(&self) -> Result<bool, FileInfoError> {
        let current_metadata = match fs::metadata(&self.path) {
            Ok(meta) => meta,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(true),
            Err(e) => {
                return Err(FileInfoError::Io {
                    path: self.path.clone(),
                    source: e,
                });
            }
        };

        let size_changed = self.file_size != current_metadata.len();
        let date_changed = self.modified_date
            != current_metadata.modified().map_err(|e| FileInfoError::Io {
                path: self.path.clone(),
                source: e,
            })?;

        Ok(size_changed || date_changed)
    }

    pub fn update(&mut self) -> Result<(), FileInfoError> {
        *self = Self::new(self.path.clone())?;
        Ok(())
    }
}
