use super::config::LibraryConfig;
use super::events::{EventCallback, LibraryEvent};
use super::metadata;
use super::scanner::DefaultScanner;
use super::storage::JsonStorage;
use super::track::{FileInfo, Track, TrackBuilder};
use super::traits::{LibraryStorage, Scanner};

use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Instant;
use std::{
    collections::{HashMap, HashSet},
    sync::atomic::{AtomicU64, Ordering},
};

use anyhow::Result;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

/// La librería principal, genérica sobre Scanner y Storage
pub struct MusicLibrary<S: Scanner, St: LibraryStorage> {
    config: LibraryConfig,
    scanner: S,
    storage: St,
    tracks: HashMap<u64, Track>,
    next_id: u64,
    callbacks: Vec<EventCallback>,
}

impl<S: Scanner, St: LibraryStorage> MusicLibrary<S, St> {
    /// Registra un listener
    pub fn on_event<F>(&mut self, callback: F)
    where
        F: FnMut(&LibraryEvent) + Send + 'static,
    {
        self.callbacks.push(Box::new(callback));
    }

    /// Llama a todos los callbacks
    fn emit(&mut self, event: LibraryEvent) {
        for cb in &mut self.callbacks {
            cb(&event);
        }
    }

    /// Refresca la librería (detecta añadidos, borrados, cambios)
    pub fn refresh_scan(&mut self) -> Result<()> {
        // Evento inicial
        self.emit(LibraryEvent::ScanStarted);

        // Tiempo total
        let start_total = Instant::now();

        // 1) Escaneo de paths
        let start_scan = Instant::now();
        let found_paths = self.scanner.scan(&self.config);
        println!("⏱ Scan de paths: {:?}", start_scan.elapsed());

        // Prepara ID y canal
        let next_id = Arc::new(AtomicU64::new(self.next_id));
        let (tx, rx) = mpsc::sync_channel::<Track>(256);

        // 2) Hilo agregador
        let start_agg = Instant::now();
        let tracks_map: Arc<Mutex<HashMap<u64, Track>>> = Arc::new(Mutex::new(HashMap::new()));
        let tracks_map_cl = Arc::clone(&tracks_map);
        let aggregator = thread::spawn(move || {
            while let Ok(track) = rx.recv() {
                let mut map = tracks_map_cl.lock().unwrap();
                map.insert(track.id, track);
            }
        });
        println!("⏱ Spawn agregador: {:?}", start_agg.elapsed());

        // 3) Worker threads
        let mut handles = Vec::new();
        for (_unit, paths) in found_paths {
            let tx_cl = tx.clone();
            let next_id_cl = Arc::clone(&next_id);

            let handle = thread::spawn(move || {
                let start_worker = Instant::now();
                for path in paths {
                    let id = next_id_cl.fetch_add(1, Ordering::Relaxed);
                    let mut builder = TrackBuilder::default();
                    let mut builder = builder.id(id).path(path.clone());

                    if let Some(track) = metadata::process(&mut builder, &path) {
                        tx_cl.send(track).expect("Error enviando track");
                    }
                }
                println!(
                    "⏱ Worker {:?}: {:?}",
                    thread::current().id(),
                    start_worker.elapsed()
                );
            });

            handles.push(handle);
        }

        // Cierra el lado de envío para que el agregador termine al recibir todo
        drop(tx);

        // 4) Espera a los workers
        let start_join = Instant::now();
        for h in handles {
            h.join().unwrap();
        }
        println!("⏱ Join workers: {:?}", start_join.elapsed());

        // 5) Espera al agregador
        let start_wait_agg = Instant::now();
        aggregator.join().unwrap();
        println!("⏱ Join agregador: {:?}", start_wait_agg.elapsed());

        // 6) Cierra y asigna resultados
        let final_map = Arc::try_unwrap(tracks_map)
            .expect("Arc aún tiene dueños")
            .into_inner()
            .unwrap();

        self.tracks = final_map;
        self.next_id = next_id.load(Ordering::Relaxed);

        // Tiempo total
        println!("✅ Full scan total: {:?}", start_total.elapsed());

        Ok(())
    }

    /// Ejemplo de full_scan (similar lógica)
    pub fn full_scan(&mut self) -> Result<()> {
        self.tracks.clear();
        self.next_id = 1;
        self.refresh_scan()
    }

    pub fn get_all_tracks(&self) -> Vec<&Track> {
        self.tracks.values().collect()
    }
}

/// Builder para MusicLibrary<S,St>
pub struct MusicLibraryBuilder<S: Scanner + Default, St: LibraryStorage + Default> {
    config: LibraryConfig,
    scanner: S,
    storage: St,
}

impl MusicLibraryBuilder<DefaultScanner, JsonStorage> {
    /// Nuevo builder con implementaciones por defecto
    pub fn new() -> Self {
        Self {
            config: LibraryConfig::default(),
            scanner: DefaultScanner::default(),
            storage: JsonStorage::default(),
        }
    }
}

impl<S: Scanner + Default, St: LibraryStorage + Default> MusicLibraryBuilder<S, St> {
    pub fn config(mut self, cfg: LibraryConfig) -> Self {
        self.config = cfg;
        self
    }

    #[allow(unused)]
    pub fn scanner<NS: Scanner + 'static + Default>(
        self,
        scanner: NS,
    ) -> MusicLibraryBuilder<NS, St> {
        MusicLibraryBuilder {
            config: self.config,
            scanner,
            storage: self.storage,
        }
    }

    pub fn storage<NS: LibraryStorage + 'static + Default>(
        self,
        storage: NS,
    ) -> MusicLibraryBuilder<S, NS> {
        MusicLibraryBuilder {
            config: self.config,
            scanner: self.scanner,
            storage,
        }
    }

    pub fn build(self) -> Result<MusicLibrary<S, St>> {
        let mut lib = MusicLibrary {
            config: self.config.clone(),
            scanner: self.scanner,
            storage: self.storage,
            tracks: HashMap::new(),
            next_id: 1,
            callbacks: Vec::new(),
        };

        let map = lib.storage.load()?;

        lib.next_id = map.keys().max().cloned().unwrap_or(0) + 1;
        lib.tracks = map;

        Ok(lib)
    }
}

// self.emit(LibraryEvent::ScanStarted);

// let start = Instant::now();

// let found_paths = self.scanner.scan(&self.config);

// let next_id = Arc::new(AtomicU64::new(self.next_id));
// let (tx, rx) = mpsc::sync_channel::<Track>(256);

// let tracks_map: Arc<Mutex<HashMap<u64, Track>>> = Arc::new(Mutex::new(HashMap::new()));
// let tracks_map_cl = Arc::clone(&tracks_map);
// let aggregator = thread::spawn(move || {
//     while let Ok(track) = rx.recv() {
//         let mut map = tracks_map_cl.lock().unwrap();
//         map.insert(track.id, track);
//     }
// });

// let mut handles = Vec::new();
// for (_unit, paths) in found_paths {
//     let tx_cl = tx.clone();
//     let next_id_cl = Arc::clone(&next_id);

//     let handle = thread::spawn(move || {
//         for path in paths {
//             // Genera ID y construye el TrackBuilder
//             let id = next_id_cl.fetch_add(1, Ordering::Relaxed);
//             let mut builder = TrackBuilder::default();
//             let mut builder = builder.id(id).path(path.clone());

//             // Procesa metadatos; si hay Track, lo envía al agregador
//             if let Some(track) = metadata::process(&mut builder, &path) {
//                 // send() bloqueará si el buffer está lleno
//                 tx_cl
//                     .send(track)
//                     .expect("Failed to send track over sync_channel");
//             }
//         }
//     });

//     handles.push(handle);
// }

// drop(tx);

// for h in handles {
//     h.join().unwrap();
// }

// aggregator.join().unwrap();

// let final_map = Arc::try_unwrap(tracks_map)
//     .expect("Arc still has multiple owners")
//     .into_inner()
//     .unwrap();

// self.tracks = final_map;
// self.next_id = next_id.load(Ordering::Relaxed);

// println!("Full scan in: {:?}", start.elapsed());

// // std::thread::spawn(move || {
// //     new_paths.par_iter().for_each(|path| {
// //         let id = next_id_clone.fetch_add(1, Ordering::Relaxed);
// //         let mut builder = TrackBuilder::default();
// //         let mut builder = builder.id(id).path(path.clone());
// //         if let Some(track) = metadata::process(&mut builder, path) {
// //             track
// //         }
// //     });
// // });

// // // 3) Construimos mapas auxiliares
// // let mut path_to_id = HashMap::with_capacity(self.tracks.len());
// // for (&id, track) in &self.tracks {
// //     path_to_id.insert(track.path.clone(), id);
// // }
// // let cached_paths: HashSet<_> = path_to_id.keys().cloned().collect();

// // // 4) Calculamos conjuntos
// // let new_paths: Vec<_> = found_paths.difference(&cached_paths).cloned().collect();
// // let existing_paths: Vec<_> = found_paths.intersection(&cached_paths).cloned().collect();
// // let removed_ids: Vec<u64> = self
// //     .tracks
// //     .iter()
// //     .filter_map(|(&id, tr)| (!found_paths.contains(&tr.path)).then(|| id))
// //     .collect();

// // // 5) Pre-reservamos el HashMap para evitar rehash
// // let extra = new_paths.len() + existing_paths.len();
// // self.tracks.reserve(extra);

// // // 6) TrackRemoved
// // for id in removed_ids {
// //     self.tracks.remove(&id);
// //     self.emit(LibraryEvent::TrackRemoved(id));
// // }

// // let (tx, rx) = unbounded::<Track>();
// // let next_id = Arc::new(AtomicU64::new(self.next_id));
// // let next_id_clone = Arc::clone(&next_id);

// // std::thread::spawn(move || {
// //     new_paths.par_iter().for_each(|path| {
// //         let id = next_id_clone.fetch_add(1, Ordering::Relaxed);
// //         let mut builder = TrackBuilder::default();
// //         let mut builder = builder.id(id).path(path.clone());
// //         if let Some(track) = metadata::process(&mut builder, path) {
// //             tx.send(track).unwrap();
// //         }
// //     });

// //     drop(tx);
// // });

// // for track in rx.iter() {
// //     self.tracks.insert(track.id, track.clone());
// //     self.emit(LibraryEvent::TrackAdded(track));
// // }

// // self.next_id = next_id.load(Ordering::Relaxed);

// // // 9) Actualizaciones: paralelas
// // let updated_tracks: Vec<Track> = existing_paths
// //     .par_iter()
// //     .filter_map(|path| {
// //         let id = path_to_id[path];
// //         let old = &self.tracks[&id];
// //         if let Some(new_info) = FileInfo::new(path) {
// //             if new_info != old.file {
// //                 let mut builder = TrackBuilder::default();
// //                 let mut builder = builder.id(id).path(path.clone());
// //                 return metadata::process(&mut builder, path);
// //             }
// //         }
// //         None
// //     })
// //     .collect();

// // // 10) Insertar y emitir TrackUpdated
// // for track in &updated_tracks {
// //     self.tracks.insert(track.id, track.clone());
// //     self.emit(LibraryEvent::TrackUpdated(track.clone()));
// // }

// // // 11) Salvado asíncrono en disco
// // let snapshot = self.tracks.clone();
// // let storage = &self.storage;
// // let _ = storage.save(&snapshot);

// // self.emit(LibraryEvent::ScanFinished);
