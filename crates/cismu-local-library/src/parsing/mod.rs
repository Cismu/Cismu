mod covers;
mod unresolved_track;

use std::borrow::Cow;
use std::{path::PathBuf, sync::Arc};

use anyhow::Result;

use cismu_paths::PATHS;

use tracing::{error, warn};

use lofty::file::TaggedFileExt;
use lofty::tag::{Accessor, ItemKey};
use lofty::{file::AudioFile, probe::Probe};

use futures::stream::FuturesUnordered;
use futures::{StreamExt, stream};
use once_cell::sync::Lazy;
use regex::Regex;
use tokio::sync::{
    Semaphore,
    mpsc::{self, Receiver, Sender},
};
use tokio::task::spawn_blocking;

use crate::parsing::covers::picture_to_cover;
use crate::scanning::{ScanResult, TrackFile};
pub use unresolved_track::UnresolvedTrack;

static COMPLEX_SEPARATORS_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)(\s*f(ea)?t(\.)?\s+)|(\s*([&×,\|])\s*)|(\s/\s)|(・)").unwrap());

/// Parsea una cadena de artista compleja en una lista de nombres limpios.
fn parse_artist_string(raw_artist: &str) -> Vec<String> {
    let standardized = COMPLEX_SEPARATORS_REGEX.replace_all(raw_artist, ";");

    standardized
        .split(';')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

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

                let result = spawn_blocking(move || Self::decode_single_audio(track, cfg.clone())).await??;

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
            // --- Metadatos de la Pista y Lanzamiento ---
            track.track_title = tag.title().map(Cow::into_owned);
            track.release_title = tag.album().map(Cow::into_owned);
            track.track_number = tag.track();
            track.disc_number = tag.disk();

            // Mapea campos adicionales usando ItemKey
            track.release_date = tag
                .get_string(&ItemKey::OriginalReleaseDate)
                .or_else(|| tag.get_string(&ItemKey::RecordingDate))
                .map(str::to_string);
            track.record_label = tag
                .get_string(&ItemKey::Publisher)
                .or_else(|| tag.get_string(&ItemKey::Label))
                .map(str::to_string);
            track.catalog_number = tag.get_string(&ItemKey::CatalogNumber).map(str::to_string);
            track.release_type = tag
                .get_string(&ItemKey::Unknown("RELEASETYPE".into()))
                .map(str::to_string);

            // Maneja géneros que pueden venir separados por '/' o ';' o ','
            track.genres = tag.genre().map(|s| {
                s.split(|c| c == '/' || c == ';' || c == ',')
                    .map(|part| part.trim().to_string())
                    .collect()
            });

            // --- Créditos ---
            if let Some(s) = tag.artist().map(Cow::into_owned) {
                track.track_performers = parse_artist_string(&s);
            }
            if let Some(s) = tag.get_string(&ItemKey::AlbumArtist) {
                track.release_artists = parse_artist_string(s);
            }
            if let Some(s) = tag.get_string(&ItemKey::Composer) {
                track.track_composers = parse_artist_string(s);
            }
            if let Some(s) = tag.get_string(&ItemKey::Producer) {
                track.track_producers = parse_artist_string(s);
            }

            // --- Re-clasificación de Artistas Invitados (Featured) ---
            if track.track_performers.len() > 1 {
                if let Some(original_artist_str) = tag.artist() {
                    let lower_artist = original_artist_str.to_lowercase();
                    // Usamos `contains` que es más flexible que buscar separadores con espacios.
                    if lower_artist.contains(" ft") || lower_artist.contains(" feat") {
                        let all_performers = track.track_performers.clone();
                        track.track_performers = vec![all_performers[0].clone()];
                        track.track_featured = all_performers[1..].to_vec();
                    }
                }
            }

            // --- Arte de Portada ---
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
                track.artworks = Some(arts);
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
