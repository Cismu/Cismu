use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use cismu_paths::UserDirs;
use jwalk::{DirEntry, WalkDir};
use rayon::ThreadPoolBuilder;
use rayon::iter::{ParallelBridge, ParallelIterator};
use serde::{Deserialize, Serialize};
use tracing::{Level, debug, debug_span, info, trace, warn};
use tracing::{info_span, instrument};

use crate::extensions::{ExtensionConfig, SupportedExtension};
use crate::traits::Scanner;

// ------------- Tipos auxiliares -------------------------------------------
#[derive(Debug, Clone)]
pub struct TrackFile {
    pub path: PathBuf,
    pub extension: SupportedExtension,
    pub file_size: u64,
    pub last_modified: u64,
}
pub type ScanResult = HashMap<String, Vec<TrackFile>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileId(u64, u64);

#[cfg(unix)]
fn file_id(path: &Path) -> Option<FileId> {
    use std::os::unix::fs::MetadataExt;
    Some(FileId(
        std::fs::metadata(path).ok()?.dev(),
        std::fs::metadata(path).ok()?.ino(),
    ))
}
#[cfg(windows)]
fn file_id(path: &Path) -> Option<FileId> {
    use std::os::windows::fs::MetadataExt;
    Some(FileId(
        std::fs::metadata(path).ok()?.volume_serial_number(),
        std::fs::metadata(path).ok()?.file_index(),
    ))
}

fn mark_seen(id: FileId, seen: &Mutex<HashSet<FileId>>) -> bool {
    let mut g = seen.lock().unwrap();
    !g.insert(id) // true ⇒ YA visto
}

// ------------- LocalScanner -----------------------------------------------
pub struct LocalScanner {
    pub config: LocalScannerConfig,
}

impl LocalScanner {
    pub fn new(config: LocalScannerConfig) -> Self {
        Self { config }
    }

    #[instrument(level = Level::DEBUG, skip_all, fields(path = %de.path().display()))]
    fn should_process_file(&self, de: DirEntry<((), ())>) -> Option<TrackFile> {
        if de.file_type().is_dir() {
            trace!(path = %de.path().display(), "Skipping directory");
            return None;
        }

        let path = de.path();
        let ext = match path.extension().and_then(OsStr::to_str) {
            Some(e) => e.to_ascii_lowercase(),
            None => {
                trace!(path = %de.path().display(), "Skipping: no extension");
                return None;
            }
        };
        let variant = SupportedExtension::from_str(&ext).ok()?;
        let ext_cfg = self.config.extensions.get(&variant).unwrap_or(&variant.config());

        let md = std::fs::metadata(&path).ok()?;
        if md.len() < ext_cfg.min_file_size.as_u64() {
            trace!(path = %de.path().display(), "Skipping: file too small");
            return None;
        }
        let mtime = md
            .modified()
            .ok()?
            .duration_since(std::time::UNIX_EPOCH)
            .ok()?
            .as_secs();

        Some(TrackFile {
            path,
            extension: variant,
            file_size: md.len(),
            last_modified: mtime,
        })
    }

    #[tracing::instrument(parent = None, skip_all, level = Level::DEBUG)]
    pub fn scan(&self) -> Result<ScanResult> {
        let seen = Arc::new(Mutex::new(HashSet::<FileId>::new()));
        let included = normalize_paths(self.config.include.clone());
        let excluded = Arc::new(normalize_paths(self.config.exclude.clone()));
        let follow = self.config.follow_symlinks;
        let mut groups: ScanResult = HashMap::new();

        info!(?included, ?self.config.exclude, threads=self.config.threads, "Starting local scan");

        let pool = ThreadPoolBuilder::new()
            .num_threads(self.config.threads)
            .build()?;

        let scan_span = info_span!("scan_pool");
        pool.install(|| {
            let _guard = scan_span.enter();

            info!("Beginning to iterate over included roots");

            for root in included {
                let root_span = info_span!("scan_root", root=%root.display());
                let _root_guard = root_span.enter();

                info!("Scanning root directory");

                //     let root_span = info_span!("scan_root", root=%root.display());
                //     if excluded.iter().any(|e| root.starts_with(e)) {
                //         debug!(root=%root.display(), "Root excluded (pre-check)");
                //         continue;
                //     }

                //     // --- 3. span por root --------------------------------------
                //     let _rsg = root_span.enter();
                //     info!("Scanning root directory");

                //     // Capture para closures
                //     let excluded_p = excluded.clone();
                //     let seen_p = seen.clone();
                //     let root_span_p = root_span.clone();

                //     let walker_iter = WalkDir::new(root.clone()).follow_links(follow).process_read_dir(
                //         move |_d, path, _state, children| {
                //             let e = info_span!(parent: &root_span_p, "walker");
                //             let _e = e.enter();

                //             // prune excluidos
                //             if excluded_p.iter().any(|e| path.starts_with(e)) {
                //                 trace!(dir=%path.display(), "Prune: excluded");
                //                 children.clear();
                //                 return;
                //             }
                //             // prune ya vistos
                //             if let Some(id) = file_id(path) {
                //                 if mark_seen(id, &*seen_p) {
                //                     trace!(dir=%path.display(), "Prune: already visited");
                //                     children.clear();
                //                 }
                //             }
                //         },
                //     );

                //     let walker = match walker_iter.try_into_iter() {
                //         Ok(w) => w,
                //         Err(e) => {
                //             warn!(error=?e, root=%root.display(), "Failed to init WalkDir");
                //             continue;
                //         }
                //     };

                //     let files: Vec<_> = walker
                //         .filter_map(|r| r.ok())
                //         .par_bridge()
                //         .filter_map(|de| root_span.in_scope(|| self.should_process_file(de)))
                //         .collect();

                //     debug!(root=%root.display(), files=files.len(), "Files collected in this root");

                //     for f in files {
                //         groups.entry(group_key(&f)).or_default().push(f);
                //     }
            }

            info!("Finished scanning all roots");
        });

        debug!(groups_total = groups.len(), "Scan complete");
        Ok(groups)
    }
}

impl Scanner for LocalScanner {}

// ------------- Helpers -----------------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalScannerConfig {
    pub include: Vec<PathBuf>,
    pub exclude: Vec<PathBuf>,
    pub follow_symlinks: bool,
    pub threads: usize,
    pub extensions: HashMap<SupportedExtension, ExtensionConfig>,
}

impl Default for LocalScannerConfig {
    fn default() -> Self {
        let include_dir = UserDirs::new()
            .and_then(|ud| ud.audio_dir().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| std::env::current_dir().unwrap());

        Self {
            include: vec!["/home/undead34/Music/Soulsheek/ALL OUT/".into()],
            exclude: vec![],
            follow_symlinks: true,
            threads: num_cpus::get(),
            extensions: HashMap::new(),
        }
    }
}

fn normalize_paths(p: Vec<PathBuf>) -> Vec<PathBuf> {
    p.into_iter()
        .filter_map(|pb| dunce::canonicalize(&pb).ok())
        .collect()
}

fn group_key(track: &TrackFile) -> String {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        std::fs::metadata(&track.path)
            .map(|m| m.dev().to_string())
            .unwrap_or_default()
    }
    #[cfg(windows)]
    {
        use std::path::Component::*;
        match track.path.components().next() {
            Some(Prefix(prefix)) => match prefix.kind() {
                Prefix::Disk(letter) | Prefix::VerbatimDisk(letter) => format!("{}:", letter as char),
                _ => "OTHER_DRIVE".into(),
            },
            _ => "NO_DRIVE".into(),
        }
    }
}
