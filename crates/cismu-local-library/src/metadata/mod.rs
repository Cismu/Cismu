mod analysis;
mod covers;
pub mod fingerprint;
mod parser;

use std::borrow::Cow;
use std::{path::PathBuf, sync::Arc};

use anyhow::Result;
use cismu_core::discography::UnresolvedTrack;
use tracing::{error, warn};

use lofty::file::TaggedFileExt;
use lofty::tag::{Accessor, ItemKey};
use lofty::{file::AudioFile, probe::Probe};

use futures::stream::FuturesUnordered;
use futures::{StreamExt, stream};
use tokio::sync::{
    Semaphore,
    mpsc::{self, Receiver, Sender},
};
use tokio::task::spawn_blocking;

use cismu_paths::PATHS;

use crate::metadata::covers::picture_to_cover;
use crate::scanner::{ScanResult, TrackFile};

#[derive(Debug, Clone)]
pub struct LocalMetadata {
    config: Arc<LocalMetadataConfig>,
}

impl LocalMetadata {
    pub fn new(config: LocalMetadataConfig) -> Self {
        LocalMetadata {
            config: config.into(),
        }
    }

    fn calc_max_threads(&self) -> (usize, usize) {
        let max_threads = (num_cpus::get() as f32 * self.config.cpu_percent / 100.0).ceil() as usize;
        let max_threads = max_threads.max(1);
        let max_threads = max_threads.clamp(1, 100);
        (max_threads, max_threads.saturating_mul(2))
    }

    pub fn process(&self, scan: ScanResult) -> Receiver<Result<UnresolvedTrack>> {
        let (max_threads, chan_size) = self.calc_max_threads();

        let (tx, rx) = mpsc::channel(chan_size);

        let config = self.config.clone();
        tokio::spawn(async move {
            let mut futs = FuturesUnordered::new();

            for (_, files) in scan.into_iter() {
                let cfg = config.clone();
                let tx = tx.clone();
                futs.push(tokio::spawn(async move {
                    Self::process_device_group(tx, files, cfg, max_threads).await
                }));
            }

            while let Some(res) = futs.next().await {
                match res {
                    Ok(Ok(())) => { /* pipeline OK */ }
                    Ok(Err(e)) => warn!(error=%e, "pipeline falló, continúo"),
                    Err(join_e) => error!(error=%join_e, "panic en pipeline! sigo"),
                }
            }

            drop(tx);
        });

        rx
    }

    /// Procesa todos los archivos de un único dispositivo en paralelo.
    async fn process_device_group(
        tx: Sender<Result<UnresolvedTrack>>,
        files: Vec<TrackFile>,
        cfg: Arc<LocalMetadataConfig>,
        permits: usize,
    ) -> Result<()> {
        let sem = Arc::new(Semaphore::new(permits));

        let stream_of_futures = files.into_iter().map(|track| {
            let sem = sem.clone();
            let cfg = cfg.clone();

            async move {
                let _permit = sem.acquire_owned().await?;

                let result =
                    spawn_blocking(move || Self::decode_single_audio(track, cfg.clone())).await??;

                Ok::<_, anyhow::Error>(result)
            }
        });

        let mut stream = stream::iter(stream_of_futures).buffer_unordered(permits);

        while let Some(result) = stream.next().await {
            if tx.send(result).await.is_err() {
                break;
            }
        }

        Ok(())
    }

    fn decode_single_audio(file: TrackFile, cfg: Arc<LocalMetadataConfig>) -> Result<UnresolvedTrack> {
        let mut track = UnresolvedTrack::default();

        // Alimentar track con TrackFile
        track.path = file.path;
        track.file_size = file.file_size;
        track.last_modified = file.last_modified;

        let tagged = Probe::open(track.path.clone())?.read()?;
        let props = tagged.properties();

        let duration = props.duration();
        let min_duration = file.extension.config().min_duration;

        if duration < min_duration {
            return Err(anyhow::anyhow!("El archivo es demasiado corto"));
        }

        // --- Detalles Técnicos ---
        track.duration = duration;
        track.bitrate_kbps = props.audio_bitrate();
        track.sample_rate = props.sample_rate();
        track.channels = props.channels();

        // --- Metadatos y Créditos ---
        if let Some(tag) = tagged.primary_tag().or_else(|| tagged.first_tag()) {
            track.title = tag.title().map(Cow::into_owned);
            track.album = tag.album().map(Cow::into_owned);
            track.track_number = tag.track().and_then(|n| n.try_into().ok());
            track.disc_number = tag.disk().and_then(|n| n.try_into().ok());
            track.genre = tag.genre().map(Cow::into_owned).map(|g| vec![g]);

            if let Some(performers_str) = tag.artist().map(Cow::into_owned) {
                let (main, featured) = parser::parse_performers(&performers_str);
                track.performers = main;
                track.featured_artists = featured;
            }

            if let Some(album_artists_str) = tag.get_string(&ItemKey::AlbumArtist) {
                track.album_artists = parser::get_raw_credits(album_artists_str);
            }

            if let Some(composers_str) = tag.get_string(&ItemKey::Composer) {
                track.composers = parser::get_raw_credits(composers_str);
            }
            if let Some(producers_str) = tag.get_string(&ItemKey::Producer) {
                track.producers = parser::get_raw_credits(producers_str);
            }

            let mut arts = Vec::new();
            for pic in tag.pictures() {
                match picture_to_cover(&pic.data(), pic.description(), cfg.cover_art_dir.clone()) {
                    Ok(art) => arts.push(art),
                    Err(e) => {
                        warn!(%e, "no se pudo procesar portada, se ignora");
                    }
                }
            }
            if !arts.is_empty() {
                track.artwork = Some(arts);
            }
        }

        Ok(track)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FingerprintAlgorithm {
    Chromaprint,
    None,
}

impl Default for FingerprintAlgorithm {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalMetadataConfig {
    pub cover_art_dir: PathBuf,
    pub fingerprint: FingerprintAlgorithm,
    /// Porcentaje de CPU a usar (0.0–100.0)
    pub cpu_percent: f32,
}

impl Default for LocalMetadataConfig {
    fn default() -> Self {
        Self {
            cover_art_dir: PATHS.covers_dir.clone(),
            fingerprint: FingerprintAlgorithm::default(),
            cpu_percent: 50.0,
        }
    }
}
