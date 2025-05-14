use super::traits::LibraryStorage;
use super::utils::Track;
use anyhow::Result;
use std::path::PathBuf;
use std::{collections::HashMap, fs};

/// Persistencia simple en JSON usando serde_json
#[derive(Debug)]
pub struct JsonStorage {
    path: PathBuf,
}

impl JsonStorage {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

impl Default for JsonStorage {
    fn default() -> Self {
        JsonStorage::new("default.db")
    }
}

impl LibraryStorage for JsonStorage {
    fn load(&self) -> Result<HashMap<u64, Track>> {
        if !self.path.exists() {
            return Ok(HashMap::new());
        }
        let data = fs::read_to_string(&self.path)?;
        let map = serde_json::from_str(&data)?;
        Ok(map)
    }

    fn save(&self, tracks: &HashMap<u64, Track>) -> Result<()> {
        let data = serde_json::to_string_pretty(tracks)?;
        fs::write(&self.path, data)?;
        Ok(())
    }
}
