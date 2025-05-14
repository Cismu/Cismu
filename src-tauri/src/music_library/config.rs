use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct LibraryConfig {
    pub database_path: PathBuf,
    pub scan_directories: Vec<PathBuf>,
    pub excluded_directories: Vec<PathBuf>,
    pub follow_symlinks: bool,
}

impl Default for LibraryConfig {
    fn default() -> Self {
        Self {
            database_path: "default.db".into(),
            scan_directories: vec![],
            excluded_directories: vec![],
            follow_symlinks: false,
        }
    }
}

impl LibraryConfig {
    pub fn builder() -> LibraryConfigBuilder {
        LibraryConfigBuilder::default()
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct LibraryConfigBuilder {
    database_path: Option<PathBuf>,
    scan_directories: Vec<PathBuf>,
    excluded_directories: Vec<PathBuf>,
    follow_symlinks: bool,
}

impl LibraryConfigBuilder {
    pub fn database_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.database_path = Some(path.into());
        self
    }

    pub fn include(mut self, path: impl Into<PathBuf>) -> Self {
        self.scan_directories.push(path.into());
        self
    }

    pub fn exclude(mut self, path: impl Into<PathBuf>) -> Self {
        self.excluded_directories.push(path.into());
        self
    }

    pub fn follow_symlinks(mut self, value: bool) -> Self {
        self.follow_symlinks = value;
        self
    }

    pub fn build(self) -> LibraryConfig {
        LibraryConfig {
            database_path: self.database_path.unwrap_or_else(|| "default.db".into()),
            scan_directories: self.scan_directories,
            excluded_directories: self.excluded_directories,
            follow_symlinks: self.follow_symlinks,
        }
    }
}
