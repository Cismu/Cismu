pub mod config_manager;
pub mod extensions;
pub mod fingerprint;
pub mod metadata;
pub mod scanner;
pub mod storage;

use std::sync::Arc;

use cismu_core::library::Library;
use tokio::runtime::Handle;
use tracing::{Level, instrument};

use crate::{
    config_manager::ConfigManager, metadata::LocalMetadata, scanner::LocalScanner, storage::LocalStorage,
};

pub struct LibraryManager {
    // scanner: Arc<LocalScanner>,
    // metadata: LocalMetadata,
    // storage: LocalStorage,
    // handle: Handle,
}

impl LibraryManager {
    #[instrument(skip_all, level = Level::DEBUG)]
    pub fn new(handle: Handle, config: ConfigManager) -> Self {
        let scanner = Arc::new(LocalScanner::new(config.scanner));
        let scanner_clone = scanner.clone();
        // let metadata = LocalMetadata::new(config.metadata);
        // let storage = LocalStorage::new(config.storage);

        // let results = handle.block_on(async move { scanner_clone.scan_async().await }).unwrap();
        // println!("Results: {:?}", results);

        LibraryManager {
            // scanner,
            // metadata,
            // storage,
            // handle,
        }
    }
}

impl Library for LibraryManager {}

// use std::path::PathBuf;

// use bliss_audio::decoder::Decoder;
// use bliss_audio::decoder::ffmpeg::FFmpegDecoder;
// use bliss_audio::playlist::{closest_to_songs, euclidean_distance};
// use bliss_audio::{BlissResult, Song};

// use crate::scanner::{LocalScanner, LocalScannerConfig};

// /// Genera una playlist inteligente basada en distancia euclidiana
// pub fn playlist_smart(paths: Vec<PathBuf>) -> BlissResult<()> {
//     println!("[INFO] Analizando canciones desde paths...");

//     let mut songs: Vec<Song> = FFmpegDecoder::analyze_paths(&paths)
//         .filter_map(|(path, res)| match res {
//             Ok(song) => {
//                 println!("[OK] Análisis exitoso: {}", path.display());
//                 Some(song)
//             }
//             Err(e) => {
//                 eprintln!("[ERROR] Falló análisis de {}: {:?}", path.display(), e);
//                 None
//             }
//         })
//         .collect();

//     if songs.is_empty() {
//         eprintln!("[ERROR] No se pudieron analizar canciones.");
//         return Ok(());
//     }

//     let first_song = songs.first().unwrap().to_owned();
//     println!("[INFO] Canción base: {}", first_song.path.display());

//     let _ = closest_to_songs(&[first_song.clone()], &mut songs, &euclidean_distance);
//     println!("[INFO] Canciones más cercanas (euclidean):");

//     println!("[INFO] Playlist generada:");
//     for song in songs {
//         println!("{}", song.path.display());
//     }

//     Ok(())
// }

// #[test]
// fn test_distance() -> BlissResult<()> {
//     let path1 = PathBuf::from("/home/undead34/Music/Soulsheek/03 - 積み木の人形.flac");
//     let path2 = PathBuf::from("/home/undead34/Music/Soulsheek/03 VILLAIN.flac");

//     println!("\n[TEST] Comparando canciones:");
//     println!(" - Song 1: {}", path1.display());
//     println!(" - Song 2: {}", path2.display());

//     let song1 = FFmpegDecoder::song_from_path(&path1)?;
//     let song2 = FFmpegDecoder::song_from_path(&path2)?;

//     let euc_dist = euclidean_distance(&song1.analysis.as_arr1(), &song2.analysis.as_arr1());

//     println!("[INFO] Euclidean distance: {:.6}", euc_dist);

//     Ok(())
// }

// #[test]
// fn test_playlist_scan() -> BlissResult<()> {
//     println!("\n[TEST] Iniciando escaneo local...");

//     let config = LocalScannerConfig::default();
//     let scanner = LocalScanner::new(config);

//     let groups = scanner.scan().expect("Fallo al escanear canciones");
//     println!("[INFO] Grupos escaneados: {}", groups.len());

//     let mut paths: Vec<PathBuf> = vec![];

//     for (group_id, songs) in &groups {
//         println!("  Grupo {}: {} canciones", group_id, songs.len());
//         for song in songs {
//             paths.push(song.path.clone());
//         }
//     }

//     if paths.is_empty() {
//         eprintln!("[WARN] No se encontraron canciones para generar playlist.");
//         return Ok(());
//     }

//     println!(
//         "[INFO] Generando playlist inteligente con {} canciones...",
//         paths.len()
//     );
//     playlist_smart(paths)?;

//     Ok(())
// }

// #[test]
// fn test_song_from_path() -> BlissResult<()> {
//     let path = PathBuf::from("/home/undead34/Music/Soulsheek/03 VILLAIN.flac");
//     let a = FFmpegDecoder::decode(&path).unwrap();
//     let s = Song::try_from(a).unwrap();
//     println!("[OK] Análisis exitoso: {:?}", s);
//     Ok(())
// }
