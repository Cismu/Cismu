pub mod audio_analysis;
pub mod config_manager;
pub mod enrichment;
pub mod library_manager;
pub mod parsing;
pub mod scanning;
pub mod storage;

pub use config_manager::ConfigManager;
pub use library_manager::LibraryManager;

// pub struct LibraryManager {
//     // scanner: Arc<LocalScanner>,
//     // metadata: Arc<LocalMetadata>,
//     // storage: Arc<Mutex<LocalStorage>>,
// }

// impl LibraryManager {
//     pub fn new(_: Handle, config: ConfigManager) -> Self {
//         // let scanner = Arc::new(LocalScanner::new(config.scanner));
//         // let metadata = Arc::new(LocalMetadata::new(config.metadata));

//         // let storage = match LocalStorage::new(config.storage) {
//         //     Ok(storage) => Arc::new(Mutex::new(storage)),
//         //     Err(e) => {
//         //         eprintln!("Failed to initialize storage: {}", e);
//         //         std::process::exit(1);
//         //     }
//         // };

//         // LibraryManager {
//         //     scanner,
//         //     metadata,
//         //     storage,
//         // }

//         let database = LocalStorage::new(config.storage).unwrap();

//         LibraryManager {}
//     }

//     pub async fn scan(&self) {
//         // let scanner = Arc::clone(&self.scanner);
//         // let metadata = Arc::clone(&self.metadata);
//         // let storage = Arc::clone(&self.storage);

//         // info!("Starting file scan...");
//         // let scan_result = scanner.scan().await.unwrap();
//         // info!("Scan complete.");

//         // // 1) Calcula cuántas pistas en total vamos a procesar:
//         // let total_tracks: usize = scan_result.iter().map(|(_, files)| files.len()).sum();

//         // // 2) Crea y configura la barra de progreso
//         // let pb = ProgressBar::new(total_tracks as u64);
//         // pb.set_draw_target(ProgressDrawTarget::stdout());
//         // pb.set_style(
//         //     ProgressStyle::default_bar()
//         //         .template(
//         //             "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
//         //         )
//         //         .unwrap()
//         //         .progress_chars("=>-"),
//         // );

//         // info!("Starting metadata processing and storage...");
//         // let start_time = Instant::now();
//         // let mut tracks_processed = 0;

//         // // let mut tracks_receiver = metadata.process(scan_result);

//         // // // 3) Por cada resultado, avanza la barra y guarda o registra el error
//         // // while let Some(result) = tracks_receiver.recv().await {
//         // //     pb.inc(1);
//         // //     match result {
//         // //         Ok(track) => {
//         // //             let artist_name: &str = track
//         // //                 .artists
//         // //                 .first()
//         // //                 .map(|s| s.as_str())
//         // //                 .unwrap_or("Unknown Artist");

//         // //             let _ = storage.lock().unwrap().insert_artist(artist_name, None);
//         // //             pb.println(format!("Processing: {}", track.file_details.path.display()));
//         // //             tracks_processed += 1;
//         // //         }
//         // //         Err(e) => {
//         // //             pb.println(format!("⚠ Failed to process: {}", e));
//         // //         }
//         // //     }
//         // // }

//         // // 4) Finaliza la barra
//         // pb.finish_with_message("Processing complete");

//         // let elapsed = start_time.elapsed();
//         // info!("Processing and storage took {} ms", elapsed.as_millis());
//         // info!("{} tracks processed and saved", tracks_processed);
//     }
// }

// impl Library for LibraryManager {}

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
