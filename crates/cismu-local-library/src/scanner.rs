use std::{
    collections::{HashMap, HashSet},
    ffi::OsStr,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
    time::UNIX_EPOCH,
};

use anyhow::Result;
use async_walkdir::{DirEntry, WalkDir};
use cismu_paths::UserDirs;
use futures::{StreamExt, TryStreamExt, future::try_join_all};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex as AsyncMutex;
use tracing::{Level, debug, info, instrument, trace, warn};

use crate::extensions::{ExtensionConfig, SupportedExtension};

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
    std::fs::metadata(path).ok().map(|m| FileId(m.dev(), m.ino()))
}
#[cfg(windows)]
fn file_id(path: &Path) -> Option<FileId> {
    use std::os::windows::fs::MetadataExt;
    std::fs::metadata(path)
        .ok()
        .map(|m| FileId(m.volume_serial_number(), m.file_index()))
}

async fn mark_seen(id: FileId, seen: &AsyncMutex<HashSet<FileId>>) -> bool {
    let mut g = seen.lock().await;
    !g.insert(id)
}

pub struct LocalScanner {
    pub config: LocalScannerConfig,
}

impl LocalScanner {
    pub fn new(config: LocalScannerConfig) -> Self {
        Self { config }
    }

    /// Escaneo asíncrono sin seguimiento de symlinks.
    pub async fn scan_async(&self) -> Result<ScanResult> {
        let seen = Arc::new(AsyncMutex::new(HashSet::<FileId>::new()));
        let included = normalize_paths(self.config.include.clone());
        let excluded = Arc::new(normalize_paths(self.config.exclude.clone()));

        info!(?included, ?self.config.exclude, "Starting local scan");

        let tasks = included.into_iter().map(|root| {
            let cfg = self.config.clone();
            let excluded = excluded.clone();
            let seen = seen.clone();

            tokio::spawn(async move { scan_root(root, cfg, excluded, seen).await })
        });

        let mut groups: ScanResult = HashMap::new();
        for res in try_join_all(tasks).await? {
            for f in res? {
                groups.entry(group_key(&f)).or_default().push(f);
            }
        }

        debug!(groups_total = groups.len(), "Scan complete");
        Ok(groups)
    }
}

#[instrument(name = "scan_root", level = Level::DEBUG, skip(cfg, excluded, seen), fields(root = %root.display()))]
async fn scan_root(
    root: PathBuf,
    cfg: LocalScannerConfig,
    excluded: Arc<Vec<PathBuf>>,
    seen: Arc<AsyncMutex<HashSet<FileId>>>,
) -> Result<Vec<TrackFile>> {
    info!(root = %root.display(), "Scanning root");
    let mut walker = WalkDir::new(root).into_stream();

    let mut found = Vec::<TrackFile>::new();

    while let Some(next) = walker.next().await {
        let de = match next {
            Ok(e) => e,
            Err(e) => {
                warn!(?e, "walkdir error");
                continue;
            }
        };

        if excluded.iter().any(|p| de.path().starts_with(p)) {
            trace!(path = %de.path().display(), "excluded");
            continue;
        }

        if let Some(id) = file_id(de.path().as_path()) {
            if mark_seen(id, &seen).await {
                trace!(path = %de.path().display(), "already seen");
                continue;
            }
        }

        if let Some(track) = should_process_file(&cfg, &de).await {
            found.push(track);
        }
    }

    Ok(found)
}

#[instrument(name = "scan_file", level = Level::TRACE, skip(cfg, de), fields(path = %de.path().display()))]
async fn should_process_file(cfg: &LocalScannerConfig, de: &DirEntry) -> Option<TrackFile> {
    if de.file_type().await.ok()?.is_dir() {
        trace!(path = %de.path().display(), "Skipping directory");
        return None;
    }

    let path = de.path();
    let ext = match path.extension().and_then(OsStr::to_str) {
        Some(e) => e.to_ascii_lowercase(),
        None => {
            trace!(path = %path.display(), "Skipping: no extension");
            return None;
        }
    };
    let variant = SupportedExtension::from_str(&ext).ok()?;
    let ext_cfg = cfg.extensions.get(&variant).unwrap_or(&variant.config());

    // metadata en Tokio (I/O no bloqueante)
    let md = tokio::fs::metadata(&path).await.ok()?;
    if md.len() < ext_cfg.min_file_size.as_u64() {
        trace!(path = %path.display(), "Skipping: file too small");
        return None;
    }
    let mtime = md.modified().ok()?.duration_since(UNIX_EPOCH).ok()?.as_secs();

    Some(TrackFile {
        path,
        extension: variant,
        file_size: md.len(),
        last_modified: mtime,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalScannerConfig {
    pub include: Vec<PathBuf>,
    pub exclude: Vec<PathBuf>,
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
                Prefix::Disk(letter) | Prefix::VerbatimDisk(letter) => {
                    format!("{}:", letter as char)
                }
                _ => "OTHER_DRIVE".into(),
            },
            _ => "NO_DRIVE".into(),
        }
    }
}
