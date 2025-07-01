use tokio::runtime::Handle;

use cismu_local_library::{
    config_manager::ConfigManager,
    metadata::LocalMetadata,
    scanner::LocalScanner,
    storage::LocalStorage,
    traits::{MetadataProcessor, Scanner, Storage},
};

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
    pub fn new(handle: Handle, config: ConfigManager) -> Self {
        let scanner = LocalScanner::new(config.scanner);
        let metadata = LocalMetadata::new(config.metadata);
        let storage = LocalStorage::new(config.storage);

        let m = handle.metrics();
        dbg!(m.num_workers());
        dbg!(m.global_queue_depth());
        dbg!(m.num_alive_tasks());
        dbg!(m.num_workers());
        dbg!(handle.runtime_flavor());
        dbg!(m);

        LibraryManager {
            scanner,
            metadata,
            storage,
            handle,
        }
    }
}
