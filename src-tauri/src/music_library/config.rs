use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
#[builder(setter(into, strip_option), default)]
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
