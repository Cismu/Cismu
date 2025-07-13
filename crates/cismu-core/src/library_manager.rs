use tokio::runtime::Handle;
use tracing::{Level, instrument};

use cismu_local_library::{
    config_manager::ConfigManager,
    traits::{MetadataProcessor, Scanner, Storage},
};

pub use cismu_local_library::{metadata::LocalMetadata, scanner::LocalScanner, storage::LocalStorage};

pub struct LibraryManager<S, M, St>
where
    S: Scanner,
    M: MetadataProcessor,
    St: Storage,
{
    scanner: S,
    metadata: M,
    storage: St,
    handle: Handle,
}

impl LibraryManager<LocalScanner, LocalMetadata, LocalStorage> {
    #[instrument(skip_all, level = Level::DEBUG)]
    pub fn new(handle: Handle, config: ConfigManager) -> Self {
        let scanner = LocalScanner::new(config.scanner);
        let metadata = LocalMetadata::new(config.metadata);
        let storage = LocalStorage::new(config.storage);

        let _ = scanner.scan();

        LibraryManager {
            scanner,
            metadata,
            storage,
            handle,
        }
    }
}
