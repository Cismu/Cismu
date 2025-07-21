use tracing::{instrument, Level};

use crate::{metadata::LocalMetadataConfig, scanner::LocalScannerConfig, storage::LocalStorageConfig};

#[derive(Debug, Clone)]
pub struct ConfigManager {
    pub scanner: LocalScannerConfig,
    pub metadata: LocalMetadataConfig,
    pub storage: LocalStorageConfig,
}

impl ConfigManager {
    #[instrument(name = "ConfigManager::new", level = Level::INFO, skip_all)]
    pub fn new() -> Self {
        ConfigManager {
            scanner: LocalScannerConfig::default(),
            metadata: LocalMetadataConfig::default(),
            storage: LocalStorageConfig::default(),
        }
    }
}
