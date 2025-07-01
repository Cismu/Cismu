use std::time::Instant;

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

        LibraryManager {
            scanner,
            metadata,
            storage,
            handle,
        }
    }
}

// let start_scan = Instant::now();
// let scanner_results = scanner.scan().unwrap();
// eprintln!("📦 Scan completed in {:?}", start_scan.elapsed());

// let start_conc = Instant::now();

// // let tracks = tokio::runtime::Runtime::new()
// //     .unwrap()
// //     .block_on(async {
// //         let results = metadata.process(scanner_results).await;
// //         results
// //     })
// //     .unwrap();

// eprintln!(
//     "🚀 Concurrent metadata processing completed in {:?}",
//     start_conc.elapsed()
// );

// println!("📦 {} tracks processed", tracks.len());
// for track in tracks.iter().take(3) {
//     println!("{:#?}", track);
// }
