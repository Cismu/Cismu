use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::Path;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::{collections::HashMap, path::PathBuf};

use anyhow::Result;
use cismu_paths::UserDirs;
use jwalk::WalkDir;
use rayon::ThreadPoolBuilder;
use rayon::iter::{ParallelBridge, ParallelIterator};
use serde::{Deserialize, Serialize};
use sugar_path::SugarPath;

use crate::extensions::{ExtensionConfig, SupportedExtension};
use crate::traits::Scanner;

#[derive(Debug, Clone)]
pub struct TrackFile {
    pub path: PathBuf,
    pub extension: SupportedExtension,
    pub file_size: u64,
    pub last_modified: u64,
}

pub type ScanResult = HashMap<String, Vec<TrackFile>>;

pub struct LocalScanner {
    config: LocalScannerConfig,
}

impl LocalScanner {
    pub fn new(config: LocalScannerConfig) -> Self {
        LocalScanner { config }
    }

    fn should_process_file(&self, dir_entry: &jwalk::DirEntry<((), ())>) -> Option<TrackFile> {
        if dir_entry.file_type().is_dir() {
            return None;
        }

        let path = dir_entry.path();

        let ext_str = match path.extension().and_then(OsStr::to_str) {
            Some(e) => e.to_ascii_lowercase(),
            None => return None,
        };

        let variant = SupportedExtension::from_str(&ext_str).ok()?;
        let ext_cfg = self.config.extensions.get(&variant).unwrap_or(&variant.config());

        let metadata = std::fs::metadata(&path).ok()?;
        if metadata.len() < ext_cfg.min_file_size.as_u64() {
            return None;
        }

        let last_modified = metadata.modified().ok()?;
        let last_modified = last_modified
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .ok()?
            .as_secs();

        Some(TrackFile {
            path: path.clone(),
            extension: variant,
            file_size: metadata.len(),
            last_modified,
        })
    }

    pub fn scan(&self) -> Result<ScanResult> {
        let seen = Arc::new(Mutex::new(HashSet::new()));
        let included = normalize_paths(self.config.include.clone());
        let excluded = Arc::new(normalize_paths(self.config.exclude.clone()));

        let follow_symlinks = self.config.follow_symlinks;

        let mut groups: ScanResult = HashMap::new();

        let pool = ThreadPoolBuilder::new()
            .num_threads(self.config.threads)
            .build()?;

        pool.install(|| {
            for root in included {
                if excluded.iter().any(|e| root.starts_with(e)) {
                    continue;
                }

                let walker = WalkDir::new(root.clone())
                    .follow_links(follow_symlinks)
                    .process_read_dir({
                        let seen = seen.clone();
                        let excluded = excluded.clone();

                        move |_depth, path, _state, children| {
                            if excluded.iter().any(|e| path.starts_with(e)) {
                                children.clear();
                                return;
                            }

                            children.retain(|entry_res| {
                                let de = match entry_res {
                                    Ok(d) => d,
                                    Err(_) => return false,
                                };

                                if !follow_symlinks && de.file_type().is_symlink() {
                                    return false;
                                }

                                let p: PathBuf = de.path();
                                let id = match file_id(&p) {
                                    Some(id) => id,
                                    None => return false,
                                };

                                !mark_seen(id, &seen)
                            });
                        }
                    });

                let collected: Vec<_> = walker
                    .into_iter()
                    .par_bridge()
                    .filter_map(|res| res.ok())
                    .filter_map(|de| self.should_process_file(&de))
                    .collect();

                for track in collected {
                    let key = group_key(&track);
                    groups.entry(key).or_default().push(track);
                }
            }
        });

        Ok(groups)
    }
}

impl Scanner for LocalScanner {}

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

        LocalScannerConfig {
            include: vec![include_dir],
            exclude: Vec::new(),
            follow_symlinks: true,
            threads: num_cpus::get(),
            extensions: HashMap::new(),
        }
    }
}

pub fn normalize_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    paths.into_iter().filter_map(normalize_path).collect()
}

pub fn normalize_path(p: PathBuf) -> Option<PathBuf> {
    let abs = p.absolutize();

    #[cfg(target_os = "windows")]
    let canon_res = dunce::canonicalize(&abs);

    #[cfg(not(target_os = "windows"))]
    let canon_res = std::fs::canonicalize(&abs);

    canon_res.ok()
}

fn group_key(track: &TrackFile) -> String {
    #[cfg(unix)]
    {
        get_unix_dev(&track.path)
    }
    #[cfg(windows)]
    {
        get_windows_group(&track.path)
    }
}

#[cfg(unix)]
fn get_unix_dev(p: &PathBuf) -> String {
    use std::os::unix::fs::MetadataExt;

    match std::fs::metadata(p) {
        Ok(md) => md.dev().to_string(),
        Err(_) => "DEV_UNKNOWN".to_string(),
    }
}

#[cfg(windows)]
fn get_windows_group(p: &PathBuf) -> String {
    todo!("To be tested in Windows");
    use std::path::Component;
    use std::path::Prefix::*;

    match p.components().next() {
        Some(Component::Prefix(prefix_comp)) => match prefix_comp.kind() {
            Disk(letter) | VerbatimDisk(letter) => format!("{}:", (letter as char)),
            UNC(_, _) | VerbatimUNC(_, _) => "UNC_OTHER".to_string(),
            _ => "OTHER_PREFIX".to_string(),
        },
        _ => "NO_DRIVE".to_string(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileId(u64, u64);

/// Convierte una ruta a un identificador único del sistema de archivos.
/// Sigue enlaces porque usa `fs::metadata`.
fn file_id(path: &Path) -> Option<FileId> {
    let md = std::fs::metadata(path).ok()?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        Some(FileId(md.dev(), md.ino()))
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        Some(FileId(md.volume_serial_number(), md.file_index()))
    }
}

/// TRUE  -> ya visto
/// FALSE -> primera vez
fn mark_seen(id: FileId, seen: &Mutex<HashSet<FileId>>) -> bool {
    let mut guard = seen.lock().unwrap();
    !guard.insert(id) // insert devuelve false si ya existía
}
