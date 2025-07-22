mod analysis;
mod covers;

use std::borrow::Cow;
use std::{path::PathBuf, sync::Arc};

use anyhow::{Result, anyhow};
use futures::stream::FuturesUnordered;

use bliss_audio::decoder::{Decoder, ffmpeg::FFmpegDecoder};
use lofty::file::TaggedFileExt;
use lofty::tag::{Accessor, ItemKey};
use lofty::{file::AudioFile, probe::Probe};

use cismu_core::discography::release_track::{AudioAnalysis, AudioQuality, FromBlissSong, UnresolvedTrack};
use cismu_paths::PATHS;

use futures::{StreamExt, stream};
use tokio::sync::{Semaphore, mpsc};
use tokio::task;

use tracing::{error, warn};

use crate::metadata::covers::picture_to_cover;
use crate::scanner::{ScanResult, TrackFile};

pub struct LocalMetadata {
    config: LocalMetadataConfig,
}

impl LocalMetadata {
    pub fn new(config: LocalMetadataConfig) -> Self {
        LocalMetadata { config }
    }

    pub fn process(&self, scan: ScanResult) -> mpsc::Receiver<Result<UnresolvedTrack>> {
        let max_threads = (num_cpus::get() as f32 * self.config.cpu_percent / 100.0).ceil() as usize;
        let max_threads = max_threads.max(1);
        let max_threads = max_threads.clamp(1, 100);

        let chan_size = max_threads.saturating_mul(2);
        let (tx, rx) = mpsc::channel(chan_size);

        let config = Arc::new(self.config.clone());
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
        tx: mpsc::Sender<std::result::Result<UnresolvedTrack, anyhow::Error>>,
        files: Vec<TrackFile>,
        config: Arc<LocalMetadataConfig>,
        max_threads: usize,
    ) -> Result<()> {
        let sem = Arc::new(Semaphore::new(max_threads));

        let stream_of_futures = files.into_iter().map(|track| {
            let sem = sem.clone();
            let config = config.clone();

            async move {
                let _permit = sem.acquire_owned().await?;

                let result =
                    task::spawn_blocking(move || Self::decode_single_audio(track, config.clone()))
                        .await??;

                Ok::<_, anyhow::Error>(result)
            }
        });

        let mut stream = stream::iter(stream_of_futures).buffer_unordered(max_threads);

        while let Some(result) = stream.next().await {
            if tx.send(result).await.is_err() {
                break;
            }
        }

        Ok(())
    }

    fn decode_single_audio(file: TrackFile, cfg: Arc<LocalMetadataConfig>) -> Result<UnresolvedTrack> {
        // let song = FFmpegDecoder::song_from_path(file.path)?;
        let mut track = UnresolvedTrack::default();
        track.file_details.path = file.path;
        let path = track.file_details.path.clone();

        let tagged = Probe::open(&path)?.read()?;
        let props = tagged.properties();

        let duration = props.duration();
        let min_duration = file.extension.config().min_duration;

        if duration < min_duration {
            return Err(anyhow!("El archivo es demasiado corto"));
        }

        let audio_bitrate = props.audio_bitrate();
        let sample_rate = props.sample_rate();
        let channels = props.channels();

        track.audio_details.duration = duration;
        track.audio_details.bitrate_kbps = audio_bitrate;
        track.audio_details.sample_rate_hz = sample_rate;
        track.audio_details.channels = channels;

        if cfg.fingerprint == FingerprintAlgorithm::Chromaprint {
            // track.audio_details.fingerprint = Some(fingerprint_from_file(&path)?);
        }

        if let Some(analysis) = track.audio_details.analysis.clone() {
            if let Some(sample_rate) = sample_rate {
                if let Some(channels) = channels {
                    let result = analysis::get_analysis(&path, sample_rate, channels);
                    match result {
                        Ok(quality) => {
                            let quality = Some(AudioQuality {
                                score: quality.quality_score,
                                assessment: quality.overall_assessment,
                            });
                            track.audio_details.analysis = Some(AudioAnalysis { quality, ..analysis });
                        }
                        Err(e) => {
                            warn!(%e, "no se pudo calcular la calidad del audio");
                        }
                    }
                }
            }
        }

        if let Some(tag) = tagged.primary_tag().or_else(|| tagged.first_tag()) {
            if track.title.is_none() {
                track.title = tag.title().map(Cow::into_owned);
            }
            if track.album.is_none() {
                track.album = tag.album().map(Cow::into_owned);
            }
            if track.album_artist.is_none() {
                track.album_artist = tag.get_string(&ItemKey::AlbumArtist).map(str::to_string);
            }
            if track.track_number.is_none() {
                track.track_number = tag.track().and_then(|n| n.try_into().ok());
            }
            if track.disc_number.is_none() {
                track.disc_number = tag.disk().and_then(|n| n.try_into().ok());
            }
            if track.genre.is_none() {
                track.genre = tag.genre().map(Cow::into_owned).map(|g| vec![g]);
            }
            if track.year.is_none() {
                track.year = tag.year().map(|y| y.to_string());
            }
            if track.composer.is_none() {
                track.composer = tag
                    .get_string(&ItemKey::Composer)
                    .map(str::to_string)
                    .map(|c| vec![c]);
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

// -----------------------------------------------------------------------------
// Config & enums
// -----------------------------------------------------------------------------
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
