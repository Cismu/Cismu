use std::{path::PathBuf, sync::Arc};

use anyhow::Result;
use bliss_audio::decoder::{Decoder, ffmpeg::FFmpegDecoder};
use cismu_core::discography::track::{FromBlissSong, UnresolvedTrack};
use cismu_paths::PATHS;
use futures::{StreamExt, future::try_join_all, stream};
use tokio::sync::{Semaphore, mpsc};
use tokio::task;
use tracing::{error, info};

use crate::scanner::{DeviceInfo, ScanResult, TrackFile};

pub fn permits_for_bw(bw_mb_s: u64) -> usize {
    match bw_mb_s {
        0..=5 => 2,    // HDD muy lento, red saturada
        6..=40 => 4,   // HDD 5400 / FUSE / Wi‑Fi
        41..=150 => 8, // HDD 7200 / red gigabit media
        _ => 16,       // SSD / NVMe
    }
}

pub struct LocalMetadata {
    config: LocalMetadataConfig,
}

impl LocalMetadata {
    pub fn new(config: LocalMetadataConfig) -> Self {
        LocalMetadata { config }
    }

    pub fn process(&self, scan: ScanResult) -> mpsc::Receiver<Result<UnresolvedTrack>> {
        let (tx, rx) = mpsc::channel(128);
        let config = Arc::new(self.config.clone());

        tokio::spawn(async move {
            let mut device_pipelines = Vec::new();

            for (device, files) in scan {
                let config = config.clone();
                let tx = tx.clone();

                let handle =
                    tokio::spawn(
                        async move { Self::process_device_group(tx, device, files, config).await },
                    );
                device_pipelines.push(handle);
            }

            match try_join_all(device_pipelines).await {
                Ok(_) => {
                    info!("Todas las pipelines de dispositivo terminaron correctamente");
                }
                Err(e) => {
                    error!("Error en alguna pipeline de dispositivo: {}", e);
                }
            }

            drop(tx);
        });

        rx
    }

    /// Procesa todos los archivos de un único dispositivo en paralelo.
    async fn process_device_group(
        tx: mpsc::Sender<std::result::Result<UnresolvedTrack, anyhow::Error>>,
        device: DeviceInfo,
        files: Vec<TrackFile>,
        config: Arc<LocalMetadataConfig>,
    ) -> Result<()> {
        let permits = permits_for_bw(device.bw_mb_s);
        let sem = Arc::new(Semaphore::new(permits));

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

        let mut stream = stream::iter(stream_of_futures).buffer_unordered(permits);

        let mut processed_count = 0;
        while let Some(result) = stream.next().await {
            processed_count += 1;
            if tx.send(result).await.is_err() {
                break;
            }
        }

        info!(
            "✅ Procesados {} archivos del dispositivo {} (concurrencia: {})",
            processed_count, device.id, permits
        );

        Ok(())
    }

    fn decode_single_audio(
        file: TrackFile,
        _config: Arc<LocalMetadataConfig>,
    ) -> Result<UnresolvedTrack> {
        let song = FFmpegDecoder::song_from_path(file.path)?;
        Ok(UnresolvedTrack::from_bliss_song(song))
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
        Self::Chromaprint
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalMetadataConfig {
    pub cover_art_dir: PathBuf,
    pub fingerprint: FingerprintAlgorithm,
}

impl Default for LocalMetadataConfig {
    fn default() -> Self {
        Self {
            cover_art_dir: PATHS.covers_dir.clone(),
            fingerprint: FingerprintAlgorithm::default(),
        }
    }
}

// // metadata.rs – versión final con autotuning por dispositivo
// // -----------------------------------------------------------------------------
// // Ahora la lógica de concurrencia por ancho de banda (`permits_for_bw`) se expone
// // como helper para que pueda ajustarse fácilmente en tests o configuraciones.

// use std::sync::Arc;
// use std::{borrow::Cow, path::PathBuf};

// use anyhow::{Result, anyhow};

// use bliss_audio::Song;
// use bliss_audio::decoder::Decoder;
// use bliss_audio::decoder::ffmpeg::FFmpegDecoder;

// use cismu_core::discography::track::{AudioAnalysis, AudioDetails, UnresolvedTrack};
// use cismu_paths::PATHS;

// use futures::{StreamExt, stream};
// use lofty::file::TaggedFileExt;
// use lofty::tag::ItemKey;
// use lofty::{file::AudioFile, probe::Probe, tag::Accessor};
// use tokio::sync::Semaphore;
// use tracing::{Instrument, Level, Span, debug, error, instrument};

// use crate::{fingerprint::fingerprint_from_file, scanner::ScanResult};

// // -----------------------------------------------------------------------------
// // Helper público para que sea configurable y testeable
// // -----------------------------------------------------------------------------
// #[inline]
// pub fn permits_for_bw(bw_mb_s: u64) -> usize {
//     match bw_mb_s {
//         0..=5 => 2,    // HDD muy lento, red saturada
//         6..=40 => 4,   // HDD 5400 / FUSE / Wi‑Fi
//         41..=150 => 8, // HDD 7200 / red gigabit media
//         _ => 16,       // SSD / NVMe
//     }
// }

// // -----------------------------------------------------------------------------
// // Estructura principal
// // -----------------------------------------------------------------------------

// pub struct LocalMetadata {
//     config: LocalMetadataConfig,
// }

// impl LocalMetadata {
//     pub fn new(config: LocalMetadataConfig) -> Self {
//         LocalMetadata { config }
//     }

//     /// Orquesta el pipeline completo: concurrencia por dispositivo (I/O) →
//     /// decodificación (CPU) → enriquecimiento de metadatos.
//     #[instrument(skip(self, scan), level = Level::INFO)]
//     pub async fn process(&self, scan: ScanResult) -> Result<Vec<UnresolvedTrack>> {
//         let mut all_tracks = Vec::new();

//         for (device, files) in scan {
//             let permits = permits_for_bw(device.bw_mb_s);
//             let sem = Arc::new(Semaphore::new(permits));

//             let stream = stream::iter(files.into_iter().map(|track| {
//                 let sem = sem.clone();
//                 let span = Span::current();
//                 async move {
//                     let _permit = sem.acquire().await?;
//                     Self::decode_single_audio(track.path).instrument(span).await
//                 }
//             }))
//             .buffer_unordered(permits);

//             for res in stream.collect::<Vec<_>>().await {
//                 match res {
//                     Ok(t) => all_tracks.push(t),
//                     Err(e) => error!(?e, "falló decodificación"),
//                 }
//             }
//         }

//         // self.process_metadata(all_tracks).await
//         Ok(all_tracks)
//     }

//     // ---------------------------------------------------------------------
//     // Decodificación (CPU‑bound)
//     // ---------------------------------------------------------------------
//     #[instrument(skip(path), level = Level::DEBUG, fields(file = %path.display()))]
//     async fn decode_single_audio(path: PathBuf) -> Result<UnresolvedTrack> {
//         let p = path.clone();
//         let track = tokio::task::spawn_blocking(move || {
//             let song: bliss_audio::Song = FFmpegDecoder::song_from_path(p)?;
//             Ok::<UnresolvedTrack, anyhow::Error>(UnresolvedTrack::from_bliss_song(song))
//         })
//         .await
//         .map_err(|e| anyhow!("Pánico en spawn_blocking: {e}"))??;

//         debug!(title = ?track.title, "Decodificación exitosa");
//         Ok(track)
//     }

//     // ---------------------------------------------------------------------
//     // Enriquecimiento de metadatos (I/O‑bound)
//     // ---------------------------------------------------------------------
//     #[instrument(skip(self, tracks), level = Level::INFO)]
//     async fn process_metadata(&self, tracks: Vec<UnresolvedTrack>) -> Result<Vec<UnresolvedTrack>> {
//         let max_concurrent = num_cpus::get().saturating_sub(1).max(1);
//         let sem = Arc::new(Semaphore::new(max_concurrent));

//         let stream = stream::iter(tracks.into_iter().map(|track| {
//             let sem = sem.clone();
//             let cfg = self.config.clone();

//             async move {
//                 let _p = sem.acquire().await?;
//                 let tagged = Probe::open(&track.path)?.read()?;
//                 let props = tagged.properties();

//                 let mut track = track;
//                 track.audio_details = AudioDetails {
//                     duration: props.duration(),
//                     bitrate_kbps: props.audio_bitrate(),
//                     sample_rate_hz: props.sample_rate(),
//                     channels: props.channels(),
//                     ..track.audio_details
//                 };

//                 if cfg.fingerprint == FingerprintAlgorithm::Chromaprint {
//                     track.audio_details.fingerprint = Some(fingerprint_from_file(&track.path)?);
//                 }

//                 if let Some(tag) = tagged.primary_tag().or_else(|| tagged.first_tag()) {
//                     if track.title.is_none() {
//                         track.title = tag.title().map(Cow::into_owned);
//                     }
//                     if track.album.is_none() {
//                         track.album = tag.album().map(Cow::into_owned);
//                     }
//                     if track.album_artist.is_none() {
//                         track.album_artist = tag.get_string(&ItemKey::AlbumArtist).map(str::to_string);
//                     }
//                     if track.track_number.is_none() {
//                         track.track_number = tag.track().and_then(|n| n.try_into().ok());
//                     }
//                     if track.disc_number.is_none() {
//                         track.disc_number = tag.disk().and_then(|n| n.try_into().ok());
//                     }
//                     if track.genre.is_none() {
//                         track.genre = tag.genre().map(Cow::into_owned).map(|g| vec![g]);
//                     }
//                     if track.year.is_none() {
//                         track.year = tag.year().map(|y| y.to_string());
//                     }
//                     if track.composer.is_none() {
//                         track.composer = tag
//                             .get_string(&ItemKey::Composer)
//                             .map(str::to_string)
//                             .map(|c| vec![c]);
//                     }
//                 }

//                 Ok::<UnresolvedTrack, anyhow::Error>(track)
//             }
//         }))
//         .buffer_unordered(max_concurrent);

//         let mut final_tracks = Vec::new();
//         for r in stream.collect::<Vec<_>>().await {
//             match r {
//                 Ok(t) => final_tracks.push(t),
//                 Err(e) => error!(?e, "error procesando metadatos"),
//             }
//         }
//         Ok(final_tracks)
//     }
// }

// mod model;

// use std::sync::Arc;
// use std::{borrow::Cow, fs, io::Cursor, path::PathBuf};

// use anyhow::{Result, anyhow};

// use bliss_audio::decoder::Decoder;
// use bliss_audio::decoder::ffmpeg::FFmpegDecoder;

// use cismu_core::discography::track::{AudioAnalysis, AudioDetails, UnresolvedTrack};
// use cismu_paths::PATHS;

// use futures::{StreamExt, stream};
// use image::{ImageReader, codecs::jpeg::JpegEncoder};
// use lofty::tag::TagItem;
// use lofty::{
//     file::{AudioFile, TaggedFileExt},
//     probe::Probe,
//     tag::{Accessor, ItemKey},
// };

// use sha2::{Digest, Sha256};
// use tokio::sync::Semaphore;
// use tracing::{Instrument, Level, Span, debug, error, info, instrument};

// use crate::{fingerprint::fingerprint_from_file, scanner::ScanResult};

// // use crate::{
// //     fingerprint::fingerprint_from_file,
// //     metadata::model::{Artwork, AudioInfo, Rating, TrackMetadata, TrackMetadataBuilder},
// //     scanner::{ScanResult, TrackFile},
// // };

// pub trait FromBlissSong<T> {
//     fn from_bliss_song(song: bliss_audio::Song) -> T;
// }

// impl FromBlissSong<UnresolvedTrack> for UnresolvedTrack {
//     fn from_bliss_song(song: bliss_audio::Song) -> Self {
//         let mut track = UnresolvedTrack::default();

//         track.path = song.path;
//         track.title = song.title;
//         track.artists = song.artist.into_iter().collect::<Vec<String>>();
//         track.album = song.album;
//         track.album_artist = song.album_artist;
//         track.track_number = song.track_number.and_then(|n| n.try_into().ok());
//         track.disc_number = song.disc_number.and_then(|n| n.try_into().ok());
//         track.genre = song.genre.map(|g| vec![g]);

//         // Style, year, statistics, composer is not available in bliss_audio::Song

//         track.audio_details.analysis = Some(AudioAnalysis {
//             features: Some(song.analysis.as_vec()),
//             ..Default::default()
//         });

//         track
//     }
// }

// pub struct LocalMetadata {
//     config: LocalMetadataConfig,
// }

// impl LocalMetadata {
//     pub fn new(config: LocalMetadataConfig) -> Self {
//         LocalMetadata { config }
//     }
// }

// impl LocalMetadata {
//     /// Calcula el número máximo de tareas concurrentes basadas en los núcleos de la CPU.
//     #[instrument(level = Level::INFO, skip_all)]
//     pub fn get_max_concurrent(&self) -> usize {
//         let cores = num_cpus::get();
//         let max_concurrent = if cores > 1 { cores - 1 } else { 1 };
//         info!(cores, max_concurrent, "Usando concurrencia limitada por CPU");
//         max_concurrent
//     }

//     /// Procesa una lista de rutas de archivos de audio en paralelo.
//     #[instrument(skip(self, all_paths), level = Level::INFO, fields(total_paths = all_paths.len()))]
//     pub async fn process_audio_details(&self, all_paths: Vec<PathBuf>) -> Result<Vec<UnresolvedTrack>> {
//         let max_concurrent = self.get_max_concurrent();
//         let semaphore = Arc::new(Semaphore::new(max_concurrent));

//         let processing_stream = stream::iter(all_paths)
//             .map(|path| {
//                 let sem_clone = semaphore.clone();
//                 let span = Span::current();

//                 async move {
//                     // Adquirimos un permiso del semáforo. Esto asegura que no se ejecuten
//                     // más de `max_concurrent` tareas de decodificación a la vez.
//                     let _permit = sem_clone
//                         .acquire()
//                         .await
//                         .map_err(|_| anyhow!("El semáforo fue cerrado inesperadamente"))?;

//                     // Procesamos el archivo dentro de su propia función instrumentada.
//                     Self::decode_single_audio(path).instrument(span).await
//                 }
//             })
//             .buffer_unordered(max_concurrent);

//         let results: Vec<Result<UnresolvedTrack>> = processing_stream.collect().await;
//         let mut all_metas = Vec::with_capacity(results.len());

//         for res in results {
//             match res {
//                 Ok(meta) => all_metas.push(meta),
//                 Err(e) => error!(error = ?e, "Error al decodificar un archivo, omitiéndolo."),
//             }
//         }

//         info!(
//             processed_count = all_metas.len(),
//             "Procesamiento de metadatos completado"
//         );

//         Ok(all_metas)
//     }

//     /// Decodifica un único archivo de audio.
//     #[instrument(skip(path), level = Level::DEBUG, fields(file = %path.display()))]
//     async fn decode_single_audio(path: PathBuf) -> Result<UnresolvedTrack> {
//         let path_clone_for_task = path.clone();

//         // `spawn_blocking` es crucial aquí para mover el trabajo intensivo en CPU
//         let song = tokio::task::spawn_blocking(move || {
//             info!("Decodificando archivo...");
//             let song: bliss_audio::Song = FFmpegDecoder::song_from_path(path_clone_for_task)?;
//             let track = UnresolvedTrack::from_bliss_song(song);

//             Ok::<UnresolvedTrack, anyhow::Error>(track)
//         })
//         .await
//         .map_err(|e| anyhow!("Pánico en spawn_blocking: {e}"))??;

//         debug!(title = ?song.title, "Decodificación exitosa");
//         Ok(song)
//     }

//     #[instrument(skip(self, tracks), level = Level::INFO)]
//     async fn process_metadata(&self, tracks: Vec<UnresolvedTrack>) -> Result<Vec<UnresolvedTrack>> {
//         let max_concurrent = self.get_max_concurrent();
//         let semaphore = Arc::new(Semaphore::new(max_concurrent));

//         let processing_stream = stream::iter(tracks)
//             .map(|track| {
//                 let sem_clone = semaphore.clone();
//                 let span = Span::current();

//                 async move {
//                     // Adquirimos un permiso del semáforo. Esto asegura que no se ejecuten
//                     // más de `max_concurrent` tareas de decodificación a la vez.
//                     let _permit = sem_clone
//                         .acquire()
//                         .await
//                         .map_err(|_| anyhow!("El semáforo fue cerrado inesperadamente"))?;

//                     // Procesamos el archivo dentro de su propia función instru
//                     let tagged_file = Probe::open(&track.path)?.read()?;
//                     let properties = tagged_file.properties();
//                     let duration = properties.duration();

//                     let mut track = track;

//                     // TODO: Implementar min_duration
//                     // if duration < self.config {
//                     //     return Err(anyhow!("El archivo es demasiado corto"));
//                     // }

//                     track.audio_details = AudioDetails {
//                         duration,
//                         bitrate_kbps: properties.audio_bitrate(),
//                         sample_rate_hz: properties.sample_rate(),
//                         channels: properties.channels(),
//                         ..track.audio_details
//                     };

//                     match self.config.fingerprint {
//                         FingerprintAlgorithm::Chromaprint => {
//                             let fingerprint = fingerprint_from_file(&track.path)?;
//                             track.audio_details.fingerprint = Some(fingerprint);
//                         }
//                         _ => {}
//                     }

//                     /// TODO: Implementar análisis de audio
//                     // pub analysis: Option<AudioAnalysis>
//                     let tag = tagged_file.primary_tag().or_else(|| tagged_file.first_tag());
//                     if let Some(tag) = tag {
//                         if let None = track.title {
//                             track.title = tag.title().map(Cow::into_owned);
//                         }

//                         if let None = track.album {
//                             track.album = tag.album().map(Cow::into_owned);
//                         }

//                         if let None = track.album_artist {
//                             track.album_artist =
//                                 tag.get_string(&ItemKey::AlbumArtist).map(str::to_string);
//                         }

//                         if let None = track.track_number {
//                             track.track_number =
//                                 tag.track().map(|n| n.try_into().ok()).unwrap_or_default();
//                         }

//                         if let None = track.disc_number {
//                             track.disc_number =
//                                 tag.disk().map(|n| n.try_into().ok()).unwrap_or_default();
//                         }

//                         if let None = track.genre {
//                             track.genre = tag.genre().map(Cow::into_owned).map(|g| vec![g]);
//                         }

//                         if let None = track.year {
//                             track.year = tag.year().map(|y| y.to_string());
//                         }

//                         if let None = track.composer {
//                             track.composer = tag
//                                 .get_string(&ItemKey::Composer)
//                                 .map(str::to_string)
//                                 .map(|c| vec![c]);
//                         }
//                     }

//                     Ok(track)
//                 }
//             })
//             .buffer_unordered(max_concurrent);

//         let results: Vec<Result<UnresolvedTrack>> = processing_stream.collect().await;
//         let mut all_metas = Vec::with_capacity(results.len());

//         for res in results {
//             match res {
//                 Ok(meta) => all_metas.push(meta),
//                 Err(e) => error!(error = ?e, "Error al decodificar un archivo, omitiéndolo."),
//             }
//         }

//         info!(
//             processed_count = all_metas.len(),
//             "Procesamiento de metadatos completado"
//         );

//         Ok(all_metas)
//     }

//     pub async fn process(&self, scan: ScanResult) -> Result<Vec<UnresolvedTrack>> {
//         let mut results = Vec::new();

//         // Procesamos por cada grupo de dispositivo
//         for (device_info, tracks) in scan {
//             let permits = match device_info.bw_mb_s as u64 {
//                 0..=5 => 2,
//                 6..=40 => 4,
//                 41..=150 => 8,
//                 _ => 16,
//             };
//             let sem = Arc::new(Semaphore::new(permits));

//             // Lanzamos procesamiento async por cada pista
//             let stream = stream::iter(tracks.into_iter().map(|t| {
//                 let sem = sem.clone();
//                 let span = Span::current();

//                 async move {
//                     let _permit = sem.acquire().await?;
//                     Self::decode_single_audio(t.path).instrument(span).await
//                 }
//             }))
//             .buffer_unordered(permits);

//             let collected: Vec<_> = stream.collect().await;

//             for r in collected {
//                 match r {
//                     Ok(track) => results.push(track),
//                     Err(e) => error!(?e, "falló decodificación"),
//                 }
//             }
//         }

//         let results = self.process_metadata(results).await?;
//         Ok(results)
//     }
// }

// #[derive(Debug, Clone, PartialEq)]
// pub enum FingerprintAlgorithm {
//     Chromaprint,
//     None,
// }

// impl Default for FingerprintAlgorithm {
//     fn default() -> Self {
//         FingerprintAlgorithm::Chromaprint
//     }
// }

// #[derive(Debug, Clone, PartialEq)]
// pub struct LocalMetadataConfig {
//     pub cover_art_dir: PathBuf,
//     pub fingerprint: FingerprintAlgorithm,
// }

// impl Default for LocalMetadataConfig {
//     fn default() -> Self {
//         LocalMetadataConfig {
//             cover_art_dir: PATHS.covers_dir.clone(),
//             fingerprint: FingerprintAlgorithm::default(),
//         }
//     }
// }

// // fn picture_to_cover(data: &[u8], description: Option<&str>, cover_art_dir: PathBuf) -> Option<Artwork> {
// //     let img = ImageReader::new(Cursor::new(data))
// //         .with_guessed_format()
// //         .ok()?
// //         .decode()
// //         .ok()?;

// //     let mut jpeg_buf = Vec::new();
// //     let mut enc = JpegEncoder::new_with_quality(&mut jpeg_buf, 100);
// //     enc.encode_image(&img.to_rgb8()).ok()?;

// //     let mut hasher = Sha256::new();
// //     hasher.update(&jpeg_buf);
// //     let hash_hex = hex::encode(hasher.finalize());

// //     let dest = cover_art_dir.join(hash_hex).with_extension("jpg");

// //     if let Some(dir) = dest.parent() {
// //         fs::create_dir_all(dir).ok()?;
// //     }

// //     if !dest.exists() {
// //         fs::write(&dest, &jpeg_buf).ok()?;
// //     }

// //     Some(Artwork {
// //         path: dest,
// //         mime_type: "image/jpeg".into(),
// //         description: description.unwrap_or_dfault().into(),
// //     })
// // }

// // pub fn process_metadata(
// //     track_file: TrackFile,
// //     config: Arc<LocalMetadataConfig>,
// // ) -> Option<TrackMetadata> {
// //     let mut metadata = TrackMetadataBuilder::default();

// //     let tagged_file = Probe::open(&track_file.path).ok()?.read().ok()?;
// //     let properties = tagged_file.properties();
// //     let duration = properties.duration();

// //     if duration < track_file.extension.config().min_duration {
// //         return None;
// //     }

// //     let mut audio_info = AudioInfo {
// //         duration_secs: duration,
// //         bitrate_kbps: properties.audio_bitrate(),
// //         sample_rate_hz: properties.sample_rate(),
// //         channels: properties.channels(),
// //         ..Default::default()
// //     };

// //     let mut tag_info = model::TagInfo::default();

// //     let tag = tagged_file.primary_tag().or_else(|| tagged_file.first_tag());

// //     if let Some(tag) = tag {
// //         audio_info.tag_type = Some(format!("{:?}", tag.tag_type()));

// //         tag_info.title = tag.title().map(Cow::into_owned);
// //         tag_info.artist = tag.artist().map(Cow::into_owned);
// //         tag_info.album = tag.album().map(Cow::into_owned);
// //         tag_info.album_artist = tag.get_string(&ItemKey::AlbumArtist).map(str::to_string);
// //         tag_info.track_number = tag.track().and_then(|n| u16::try_from(n).ok());
// //         tag_info.total_tracks = tag.track_total().map(|n| n as u16).or_else(|| {
// //             tag.get_string(&ItemKey::TrackTotal)
// //                 .and_then(|s| s.trim().parse::<u16>().ok())
// //         });
// //         tag_info.disc_number = tag.disk().and_then(|n| u16::try_from(n).ok());
// //         tag_info.total_discs = tag.disk_total().map(|n| n as u16).or_else(|| {
// //             tag.get_string(&ItemKey::DiscTotal)
// //                 .and_then(|s| s.trim().parse::<u16>().ok())
// //         });
// //         tag_info.genre = tag.genre().map(Cow::into_owned);
// //         tag_info.year = tag.year();
// //         tag_info.composer = tag.get_string(&ItemKey::Composer).map(str::to_string);
// //         tag_info.publisher = tag.get_string(&ItemKey::Publisher).map(str::to_string);
// //         tag_info.comments = tag.comment().map(Cow::into_owned);
// //         tag_info.rating = Rating::from_tag(tag);

// //         let mut arts = Vec::new();

// //         for pic in tag.pictures() {
// //             if let Some(art) =
// //                 picture_to_cover(&pic.data(), pic.description(), config.cover_art_dir.clone())
// //             {
// //                 arts.push(art);
// //             }
// //         }

// //         if !arts.is_empty() {
// //             tag_info.artwork = Some(arts);
// //         }
// //     }

// //     if config.fingerprint == FingerprintAlgorithm::Chromaprint {
// //         audio_info.fingerprint = fingerprint_from_file(&track_file.path).ok();
// //     }

// //     metadata.tags(tag_info);

// //     metadata.file(model::FileInfo::new(track_file));
// //     metadata.audio(audio_info);
// //     metadata.build().ok()
// // }
