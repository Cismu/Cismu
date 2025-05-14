use super::config::LibraryConfig;
use super::events::{EventCallback, LibraryEvent};
use super::metadata::{self, TrackMetadata};
use super::scanner::DefaultScanner;
use super::storage::JsonStorage;
use super::traits::{LibraryStorage, Scanner};
use super::utils::Track;

use anyhow::Result;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{
    collections::{HashMap, HashSet},
    fs,
    sync::atomic::{AtomicU64, Ordering},
    time::UNIX_EPOCH,
};

/// La librería principal, genérica sobre Scanner y Storage
pub struct MusicLibrary<S: Scanner, St: LibraryStorage> {
    config: LibraryConfig,
    scanner: S,
    storage: St,
    tracks: HashMap<u64, Track>,
    next_id: u64,
    callbacks: Vec<EventCallback<'static>>,
}

/// Builder para MusicLibrary<S,St>
pub struct MusicLibraryBuilder<S: Scanner + Default, St: LibraryStorage + Default> {
    config: LibraryConfig,
    scanner: S,
    storage: St,
    callbacks: Vec<EventCallback<'static>>,
}

impl MusicLibraryBuilder<DefaultScanner, JsonStorage> {
    /// Nuevo builder con implementaciones por defecto
    pub fn new() -> Self {
        Self {
            config: LibraryConfig::default(),
            scanner: DefaultScanner::default(),
            storage: JsonStorage::default(),
            callbacks: vec![],
        }
    }
}

impl<S: Scanner + Default, St: LibraryStorage + Default> MusicLibraryBuilder<S, St> {
    pub fn config(mut self, cfg: LibraryConfig) -> Self {
        self.config = cfg;
        self
    }

    pub fn scanner<NS: Scanner + 'static + Default>(
        self,
        scanner: NS,
    ) -> MusicLibraryBuilder<NS, St> {
        MusicLibraryBuilder {
            config: self.config,
            scanner,
            storage: self.storage,
            callbacks: self.callbacks,
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
            callbacks: self.callbacks,
        }
    }

    pub fn on_event<F>(mut self, cb: F) -> Self
    where
        F: FnMut(LibraryEvent) + Send + 'static,
    {
        self.callbacks.push(Box::new(cb));
        self
    }

    pub fn build(self) -> Result<MusicLibrary<S, St>> {
        let mut lib = MusicLibrary {
            config: self.config.clone(),
            scanner: self.scanner,
            storage: self.storage,
            tracks: HashMap::new(),
            next_id: 1,
            callbacks: self.callbacks,
        };

        // Cargar del storage si existe
        match lib.storage.load() {
            Ok(map) => {
                lib.next_id = map.keys().max().cloned().unwrap_or(0) + 1;
                lib.tracks = map;
            }
            Err(e) => lib.emit(LibraryEvent::Error(&e)),
        }

        Ok(lib)
    }
}

impl<S: Scanner, St: LibraryStorage> MusicLibrary<S, St> {
    /// Registro de callbacks tras crear la librería
    pub fn on_event<F>(&mut self, cb: F)
    where
        F: FnMut(LibraryEvent) + Send + 'static,
    {
        self.callbacks.push(Box::new(cb));
    }

    fn emit(&mut self, ev: LibraryEvent) {
        // for cb in &mut self.callbacks {
        //     cb(ev);
        // }
    }

    /// Refresca la librería (detecta añadidos, borrados, cambios)
    pub fn refresh_scan(&mut self) -> Result<()> {
        self.emit(LibraryEvent::ScanStarted);

        // let current = self.scanner.scan(&self.config);
        // let mut path_to_id = HashMap::new();
        // let mut cached = HashSet::new();

        // for (id, tr) in &self.tracks {
        //     path_to_id.insert(tr.path.clone(), *id);
        //     cached.insert(tr.path.clone());
        // }

        // let new_paths: HashSet<_> = current.difference(&cached).cloned().collect();
        // let del_paths: HashSet<_> = cached.difference(&current).cloned().collect();
        // let existing: Vec<_> = current.intersection(&cached).cloned().collect();

        // // Borrados
        // for p in &del_paths {
        //     if let Some(&id) = path_to_id.get(p) {
        //         self.tracks.remove(&id);
        //         self.emit(LibraryEvent::TrackRemoved(id));
        //     }
        // }

        // // Nuevos
        // let next_atomic = AtomicU64::new(self.next_id);
        // let new_tracks: Vec<_> = new_paths
        //     .par_iter()
        //     .filter_map(|p| {
        //         metadata::process(p).map(|m| {
        //             let id = next_atomic.fetch_add(1, Ordering::SeqCst);
        //             Track {
        //                 id,
        //                 path: p.clone(),
        //                 metadata: m,
        //             }
        //         })
        //     })
        //     .collect();
        // self.next_id = next_atomic.load(Ordering::SeqCst);

        // for tr in new_tracks {
        //     let id = tr.id;
        //     self.tracks.insert(id, tr);
        //     self.emit(LibraryEvent::TrackAdded(self.tracks.get(&id).unwrap()));
        // }

        // // Actualizaciones
        // for p in existing {
        //     if let Some(&id) = path_to_id.get(&p) {
        //         if let Some(old) = self.tracks.get(&id) {
        //             if let Ok(md) = fs::metadata(&p) {
        //                 let size = md.len();
        //                 let mtime = md
        //                     .modified()
        //                     .ok()
        //                     .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        //                     .map(|d| d.as_secs());
        //                 if Some(size) != Some(old.metadata.file_size())
        //                     || Some(mtime.unwrap_or(0)) != Some(old.metadata.last_modification())
        //                 {
        //                     if let Some(new_meta) = metadata::process(&p) {
        //                         let tr_mut = self.tracks.get_mut(&id).unwrap();
        //                         tr_mut.metadata = new_meta;
        //                         self.emit(LibraryEvent::TrackUpdated(tr_mut));
        //                     }
        //                 }
        //             }
        //         }
        //     }
        // }

        self.emit(LibraryEvent::ScanFinished);
        self.storage.save(&self.tracks)?;
        Ok(())
    }

    /// Ejemplo de full_scan (similar lógica)
    pub fn full_scan(&mut self) -> Result<()> {
        self.tracks.clear();
        self.next_id = 1;
        self.refresh_scan()
    }
}
