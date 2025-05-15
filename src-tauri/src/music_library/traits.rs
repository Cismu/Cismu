use super::{config::LibraryConfig, track::Track};
use anyhow::Result;
use std::{collections::HashMap, collections::HashSet, path::PathBuf};

/// Trait para abstraer la lógica de escaneo de archivos
pub trait Scanner {
    fn scan(&self, config: &LibraryConfig) -> HashSet<PathBuf>;
}

/// Trait para abstraer la persistencia de la biblioteca
pub trait LibraryStorage {
    fn load(&self) -> Result<HashMap<u64, Track>>;
    fn save(&self, tracks: &HashMap<u64, Track>) -> Result<()>;
}
