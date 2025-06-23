use config::{Config, File, FileFormat};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::{
    error::ConfigError,
    extensions::{ExtensionConfig, SupportedExtension, default_extension_config},
};

/// Backends de base de datos
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "path")]
pub enum DatabaseBackend {
    Sqlite(PathBuf),
}

impl Default for DatabaseBackend {
    fn default() -> Self {
        DatabaseBackend::Sqlite("library.db".into())
    }
}

/// Algoritmos de fingerprinting
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FingerprintAlgorithm {
    Chromaprint,
}

impl Default for FingerprintAlgorithm {
    fn default() -> Self {
        FingerprintAlgorithm::Chromaprint
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
#[builder(setter(into, strip_option), default)]
pub struct LibraryConfig {
    pub database: DatabaseBackend,
    pub include_paths: Vec<PathBuf>,
    pub exclude_paths: Vec<PathBuf>,
    pub follow_symlinks: bool,
    pub extension_config: std::collections::HashMap<SupportedExtension, ExtensionConfig>,
    pub cover_art_dir: Option<PathBuf>,
    pub fingerprint_algorithm: FingerprintAlgorithm,
    pub scan_threads: Option<usize>,
}

impl Default for LibraryConfig {
    fn default() -> Self {
        LibraryConfig {
            database: DatabaseBackend::Sqlite("library.db".into()),
            include_paths: Vec::new(),
            exclude_paths: Vec::new(),
            follow_symlinks: true,
            extension_config: default_extension_config(),
            cover_art_dir: Some(PathBuf::from("cover_art")),
            fingerprint_algorithm: FingerprintAlgorithm::Chromaprint,
            scan_threads: None,
        }
    }
}

impl LibraryConfig {
    pub fn from_file(path: impl AsRef<std::path::Path>) -> Result<Self, ConfigError> {
        let path = path.as_ref().to_string_lossy().into_owned();
        let cfg = Config::builder()
            .add_source(File::new(&path, FileFormat::Toml))
            .build()
            .map_err(ConfigError::Parse)?;
        let lc = cfg
            .try_deserialize::<LibraryConfig>()
            .map_err(ConfigError::Parse)?;
        Ok(lc)
    }
}
