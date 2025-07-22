use anyhow::Result;
use tracing::error;

use crate::{
    config_manager::ConfigManager, metadata::LocalMetadata, scanner::LocalScanner, storage::LocalStorage,
};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct LibraryManager {
    scanner: Arc<LocalScanner>,
    metadata: Arc<LocalMetadata>,
    storage: Arc<LocalStorage>,
}

impl LibraryManager {
    pub fn new(config: ConfigManager) -> Self {
        let scanner = LocalScanner::new(config.scanner);
        let metadata = LocalMetadata::new(config.metadata);
        let storage = LocalStorage::new(config.storage).unwrap();

        Self {
            scanner: Arc::new(scanner),
            metadata: Arc::new(metadata),
            storage: Arc::new(storage),
        }
    }

    pub async fn scan(&self) -> Result<()> {
        let scanner = Arc::clone(&self.scanner);
        let metadata = Arc::clone(&self.metadata);
        let storage = Arc::clone(&self.storage);

        let groups = scanner.scan().await?;
        let mut rx = metadata.process(groups);

        while let Some(res) = rx.recv().await {
            match res {
                Ok(track) => {
                    storage.resolve_unresolved_track(track);
                }
                Err(e) => {
                    error!(%e, "error al procesar metadata");
                }
            }
        }

        Ok(())
    }
}

impl Default for LibraryManager {
    fn default() -> Self {
        let config = ConfigManager::default();
        let scanner = LocalScanner::new(config.scanner);
        let metadata = LocalMetadata::new(config.metadata);
        let storage = LocalStorage::new(config.storage).unwrap();

        Self {
            scanner: Arc::new(scanner),
            metadata: Arc::new(metadata),
            storage: Arc::new(storage),
        }
    }
}
