mod extensions;

use std::{
    collections::{HashMap, HashSet},
    ffi::OsStr,
    io::Read,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
    time::UNIX_EPOCH,
};

use anyhow::{Context, Result};
use async_walkdir::{DirEntry, WalkDir};
use cismu_paths::UserDirs;
use futures::{StreamExt, TryStreamExt, future::try_join_all};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex as AsyncMutex;
use tracing::{Level, instrument, warn};

use extensions::{ExtensionConfig, SupportedExtension};

/// Métricas de dispositivo descubiertas dinámicamente.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DeviceInfo {
    pub id: String,   // dev_t en Unix, letra de unidad en Windows, etc.
    pub bw_mb_s: u64, // ancho de banda estimado (MB/s)
}

#[derive(Debug, Clone)]
pub struct TrackFile {
    pub path: PathBuf,
    pub extension: SupportedExtension,
    pub file_size: u64,
    pub last_modified: u64,
}

/// Resultado final: para cada dispositivo, lista de pistas + métricas
pub type ScanResult = HashMap<DeviceInfo, Vec<TrackFile>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct FileId(u64, u64);

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

/// Lee `sample_bytes` del principio del archivo y devuelve MB/s.
fn measure_device_throughput(sample_path: &Path, sample_bytes: usize) -> Result<f64> {
    let start = std::time::Instant::now();
    let mut file =
        std::fs::File::open(sample_path).with_context(|| format!("open {}", sample_path.display()))?;
    let mut buf = vec![0u8; sample_bytes];
    let mut read_total = 0usize;
    while read_total < sample_bytes {
        let n = file.read(&mut buf[read_total..])?;
        if n == 0 {
            break; // EOF
        }
        read_total += n;
    }
    let secs = start.elapsed().as_secs_f64();
    Ok(if secs == 0.0 {
        0.0
    } else {
        (read_total as f64) / 1_048_576.0 / secs
    }) // MB/s
}

fn normalize_paths(p: Vec<PathBuf>) -> Vec<PathBuf> {
    p.into_iter()
        .filter_map(|pb| dunce::canonicalize(&pb).ok())
        .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalScannerConfig {
    pub include: Vec<PathBuf>,
    pub exclude: Vec<PathBuf>,
    pub extensions: HashMap<SupportedExtension, ExtensionConfig>,
    /// Bytes que se leen para estimar el BW (por defecto 3 MiB)
    pub sample_bytes: usize,
}

impl Default for LocalScannerConfig {
    fn default() -> Self {
        let include_dir = UserDirs::new()
            .and_then(|ud| ud.audio_dir().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| std::env::current_dir().unwrap());

        Self {
            include: vec![include_dir],
            exclude: vec![],
            extensions: HashMap::new(),
            sample_bytes: 3 * 1_048_576,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LocalScanner {
    pub config: LocalScannerConfig,
}

impl LocalScanner {
    pub fn new(config: LocalScannerConfig) -> Self {
        Self { config }
    }

    /// Realiza el escaneo y devuelve los grupos por dispositivo + métricas de BW.
    #[instrument(level = Level::INFO, skip(self))]
    pub async fn scan(&self) -> Result<ScanResult> {
        let seen = Arc::new(AsyncMutex::new(HashSet::<FileId>::new()));
        let included = normalize_paths(self.config.include.clone());
        let excluded = Arc::new(normalize_paths(self.config.exclude.clone()));

        let tasks = included.into_iter().map(|root| {
            let cfg = self.config.clone();
            let excluded = excluded.clone();
            let seen = seen.clone();
            tokio::spawn(scan_root(root, cfg, excluded, seen))
        });

        let mut tmp: HashMap<String, Vec<TrackFile>> = HashMap::new();
        for res in try_join_all(tasks).await? {
            for track in res? {
                let dev_id = device_id(&track.path)?;
                tmp.entry(dev_id).or_default().push(track);
            }
        }

        // Autotune: medir BW para cada dev_id en paralelo (blocking OK, pocos dispositivos)
        let mut scan_result: ScanResult = HashMap::new();
        let mut bw_handles = Vec::new();
        for (dev_id, tracks) in tmp.into_iter() {
            if tracks.is_empty() {
                continue;
            }

            let sample_path = tracks[0].path.clone();
            let bytes = self.config.sample_bytes;
            bw_handles.push(tokio::task::spawn_blocking(move || {
                let bw = measure_device_throughput(&sample_path, bytes).unwrap_or(0.0);
                (dev_id, bw, tracks)
            }));
        }

        for h in bw_handles {
            let (dev_id, bw, tracks) = h.await?;
            let di = DeviceInfo {
                id: dev_id,
                bw_mb_s: bw as u64,
            };
            scan_result.insert(di, tracks);
        }

        Ok(scan_result)
    }
}

// Dev‑id helpers --------------------------------------------------------------
#[cfg(unix)]
fn device_id(path: &Path) -> Result<String> {
    use std::os::unix::fs::MetadataExt;
    let meta = std::fs::metadata(path)?;
    Ok(meta.dev().to_string())
}

#[cfg(windows)]
fn device_id(path: &Path) -> Result<String> {
    use std::path::Component;
    let drive = match path.components().next() {
        Some(Component::Prefix(prefix)) => match prefix.kind() {
            std::path::Prefix::Disk(letter) | std::path::Prefix::VerbatimDisk(letter) => {
                format!("{}:", letter as char)
            }
            _ => "OTHER_DRIVE".into(),
        },
        _ => "NO_DRIVE".into(),
    };
    Ok(drive)
}

// Scan raíz --------------------------------------------------------------
async fn scan_root(
    root: PathBuf,
    cfg: LocalScannerConfig,
    excluded: Arc<Vec<PathBuf>>,
    seen: Arc<AsyncMutex<HashSet<FileId>>>,
) -> Result<Vec<TrackFile>> {
    let mut walker = WalkDir::new(root).into_stream();
    let mut found = Vec::new();

    while let Some(next) = walker.next().await {
        match next {
            Ok(de) => {
                let path = de.path().to_path_buf();
                if excluded.iter().any(|p| path.starts_with(p)) {
                    continue;
                }

                if let Some(id) = file_id(&path) {
                    if mark_seen(id, &seen).await {
                        continue;
                    }
                }

                if let Some(track) = should_process_file(&cfg, &de).await {
                    found.push(track);
                }
            }
            Err(e) => warn!(?e, "walkdir error"),
        }
    }

    Ok(found)
}

async fn should_process_file(cfg: &LocalScannerConfig, de: &DirEntry) -> Option<TrackFile> {
    if de.file_type().await.ok()?.is_dir() {
        return None;
    }

    let path = de.path().to_path_buf();
    let ext = path.extension().and_then(OsStr::to_str)?.to_ascii_lowercase();
    let variant = SupportedExtension::from_str(&ext).ok()?;
    let ext_cfg = cfg.extensions.get(&variant).unwrap_or(&variant.config());

    let md = tokio::fs::metadata(&path).await.ok()?;
    if md.len() < ext_cfg.min_file_size.as_u64() {
        return None;
    }

    let last_modified = md
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())?;

    Some(TrackFile {
        path,
        extension: variant,
        file_size: md.len(),
        last_modified,
    })
}
