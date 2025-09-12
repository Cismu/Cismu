use std::sync::Arc;

use anyhow::Result;
use cismu_core::discography::{
    artist::{Artist, ArtistId},
    release::{Release, ReleaseId},
};
use tracing::error;

use crate::{config_manager::ConfigManager, parsing::LocalMetadata, scanning::LocalScanner, storage::LocalStorage};

#[derive(Debug, Clone)]
pub struct LibraryManager {
    scanner: Arc<LocalScanner>,
    metadata: Arc<LocalMetadata>,
    // storage: Arc<LocalStorage>,
}

impl LibraryManager {
    pub fn new(config: ConfigManager) -> Self {
        let scanner = LocalScanner::new(config.scanner);
        let metadata = LocalMetadata::new(config.metadata);
        // let storage = LocalStorage::new();

        Self {
            scanner: Arc::new(scanner),
            metadata: Arc::new(metadata),
            // storage: Arc::new(storage),
        }
    }

    pub async fn scan(&self) -> Result<()> {
        let scanner = Arc::clone(&self.scanner);
        let metadata = Arc::clone(&self.metadata);

        let groups = scanner.scan().await?;
        let mut rx = metadata.process(groups);

        while let Some(res) = rx.recv().await {
            match res {
                Ok(track) => {
                    // storage.resolve_unresolved_track(track)?;
                }
                Err(e) => {
                    error!(%e, "error al procesar metadata");
                }
            }
        }

        Ok(())
    }

    // pub fn get_all_artists(&self) -> Result<Vec<Artist>> {
    //     self.storage.get_all_artists()
    // }

    // pub fn get_releases_for_artist(&self, artist_id: ArtistId) -> Result<Vec<Release>> {
    //     self.storage.get_releases_for_artist(artist_id)
    // }

    // pub fn get_release_details(&self, release_id: ReleaseId) -> Result<Option<Release>> {
    //     self.storage.get_release_details(release_id)
    // }
}

impl Default for LibraryManager {
    fn default() -> Self {
        let config = ConfigManager::default();
        let scanner = LocalScanner::new(config.scanner);
        let metadata = LocalMetadata::new(config.metadata);
        // let storage = LocalStorage::new(config.storage).unwrap();

        Self {
            scanner: Arc::new(scanner),
            metadata: Arc::new(metadata),
            // storage: Arc::new(storage),
        }
    }
}

// API Key for submit fingerprint to AcoustID: 2igXkDlTE9
// API Key for use AcoustID: atU38bBrIw
