use super::config::LibraryConfig;
use super::metadata;
use super::scanner::DefaultScanner;
use super::storage::JsonStorage;
use super::track::{FileInfo, Track, TrackBuilder};
use super::traits::{LibraryStorage, Scanner};

use anyhow::Result;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{
    collections::{HashMap, HashSet},
    sync::atomic::{AtomicU64, Ordering},
};

/// La librería principal, genérica sobre Scanner y Storage
pub struct MusicLibrary<S: Scanner, St: LibraryStorage> {
    config: LibraryConfig,
    scanner: S,
    storage: St,
    tracks: HashMap<u64, Track>,
    next_id: u64,
}

impl<S: Scanner, St: LibraryStorage> MusicLibrary<S, St> {
    /// Refresca la librería (detecta añadidos, borrados, cambios)
    pub fn refresh_scan(&mut self) -> Result<()> {
        let current = self.scanner.scan(&self.config);
        let mut path_to_id = HashMap::new();
        let mut cached = HashSet::new();

        for (id, tr) in &self.tracks {
            path_to_id.insert(tr.path.clone(), *id);
            cached.insert(tr.path.clone());
        }

        let new_paths: HashSet<_> = current.difference(&cached).cloned().collect();
        let del_paths: HashSet<_> = cached.difference(&current).cloned().collect();
        let existing: Vec<_> = current.intersection(&cached).cloned().collect();

        // Borrados
        for p in &del_paths {
            if let Some(&id) = path_to_id.get(p) {
                self.tracks.remove(&id);
            }
        }

        // Nuevos
        let next_atomic = AtomicU64::new(self.next_id);
        let new_tracks: Vec<_> = new_paths
            .par_iter()
            .filter_map(|p| {
                let id = next_atomic.fetch_add(1, Ordering::Relaxed);
                let mut track = TrackBuilder::default();
                track.id(id).path(p.clone());
                metadata::process(&mut track, p)
            })
            .collect();

        self.next_id = next_atomic.load(Ordering::Relaxed);

        for tr in new_tracks {
            let id = tr.id;
            self.tracks.insert(id, tr);
        }

        // Actualizaciones
        for p in existing {
            if let Some(&id) = path_to_id.get(&p) {
                if let Some(old) = self.tracks.get(&id) {
                    if let Some(new_info) = FileInfo::new(&p) {
                        if new_info != old.file {
                            let mut track = TrackBuilder::default();
                            track.id(id).path(p.clone());
                            if let Some(track) = metadata::process(&mut track, &p) {
                                self.tracks.insert(id, track);
                            }
                        }
                    }
                }
            }
        }

        self.storage.save(&self.tracks)?;
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
        };

        let map = lib.storage.load()?;

        lib.next_id = map.keys().max().cloned().unwrap_or(0) + 1;
        lib.tracks = map;

        Ok(lib)
    }
}
