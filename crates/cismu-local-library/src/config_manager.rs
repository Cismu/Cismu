use crate::{metadata::LocalMetadataConfig, scanner::LocalScannerConfig, storage::LocalStorageConfig};

#[derive(Debug, Clone)]
pub struct ConfigManager {
    pub scanner: LocalScannerConfig,
    pub metadata: LocalMetadataConfig,
    pub storage: LocalStorageConfig,
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self {
            scanner: LocalScannerConfig::default(),
            metadata: LocalMetadataConfig::default(),
            storage: LocalStorageConfig::default(),
        }
    }
}
