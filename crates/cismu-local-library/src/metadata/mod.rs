mod model;

use std::{borrow::Cow, fs, io::Cursor, path::PathBuf};

use anyhow::{Result, anyhow};
use bliss_audio::Song;
use cismu_paths::PATHS;
use futures::{StreamExt, stream::FuturesUnordered};
use image::{ImageReader, codecs::jpeg::JpegEncoder};
use lofty::{
    file::{AudioFile, TaggedFileExt},
    probe::Probe,
    tag::{Accessor, ItemKey},
};
use sha2::{Digest, Sha256};
use tokio::{runtime::Handle, sync::mpsc};

use crate::{
    fingerprint::fingerprint_from_file,
    metadata::model::{Artwork, AudioInfo, Rating, TrackMetadata, TrackMetadataBuilder},
    scanner::{ScanResult, TrackFile},
};

pub struct LocalMetadata {
    config: LocalMetadataConfig,
    handle: Handle,
}

impl LocalMetadata {
    pub fn new(handle: Handle, config: LocalMetadataConfig) -> Self {
        LocalMetadata { config, handle }
    }

    pub fn process_metadata(
        track_file: TrackFile,
        config: LocalMetadataConfig,
    ) -> Option<TrackMetadata> {
        let mut metadata = TrackMetadataBuilder::default();

        let tagged_file = Probe::open(&track_file.path).ok()?.read().ok()?;
        let properties = tagged_file.properties();
        let duration = properties.duration();

        if duration < track_file.extension.config().min_duration {
            return None;
        }

        let mut audio_info = AudioInfo {
            duration_secs: duration,
            bitrate_kbps: properties.audio_bitrate(),
            sample_rate_hz: properties.sample_rate(),
            channels: properties.channels(),
            ..Default::default()
        };

        let mut tag_info = model::TagInfo::default();

        let tag = tagged_file.primary_tag().or_else(|| tagged_file.first_tag());

        if let Some(tag) = tag {
            audio_info.tag_type = Some(format!("{:?}", tag.tag_type()));

            tag_info.title = tag.title().map(Cow::into_owned);
            tag_info.artist = tag.artist().map(Cow::into_owned);
            tag_info.album = tag.album().map(Cow::into_owned);
            tag_info.album_artist = tag.get_string(&ItemKey::AlbumArtist).map(str::to_string);
            tag_info.track_number = tag.track().and_then(|n| u16::try_from(n).ok());
            tag_info.total_tracks = tag.track_total().map(|n| n as u16).or_else(|| {
                tag.get_string(&ItemKey::TrackTotal)
                    .and_then(|s| s.trim().parse::<u16>().ok())
            });
            tag_info.disc_number = tag.disk().and_then(|n| u16::try_from(n).ok());
            tag_info.total_discs = tag.disk_total().map(|n| n as u16).or_else(|| {
                tag.get_string(&ItemKey::DiscTotal)
                    .and_then(|s| s.trim().parse::<u16>().ok())
            });
            tag_info.genre = tag.genre().map(Cow::into_owned);
            tag_info.year = tag.year();
            tag_info.composer = tag.get_string(&ItemKey::Composer).map(str::to_string);
            tag_info.publisher = tag.get_string(&ItemKey::Publisher).map(str::to_string);
            tag_info.comments = tag.comment().map(Cow::into_owned);
            tag_info.rating = Rating::from_tag(tag);

            let mut arts = Vec::new();

            for pic in tag.pictures() {
                if let Some(art) =
                    picture_to_cover(&pic.data(), pic.description(), config.cover_art_dir.clone())
                {
                    arts.push(art);
                }
            }

            if !arts.is_empty() {
                tag_info.artwork = Some(arts);
            }
        }

        if config.fingerprint == FingerprintAlgorithm::Chromaprint {
            audio_info.fingerprint = fingerprint_from_file(&track_file.path).ok();
        }

        metadata.tags(tag_info);

        metadata.file(model::FileInfo::new(track_file));
        metadata.audio(audio_info);
        metadata.build().ok()
    }
}

impl LocalMetadata {
    pub async fn process(&self, scan: ScanResult) -> Result<Vec<TrackMetadata>> {
        let (tx, mut rx) = mpsc::channel(512);

        for track_files in scan.into_values() {
            let tx = tx.clone();
            let config = self.config.clone();

            tokio::spawn(async move {
                let mut futures = FuturesUnordered::new();

                for track_file in track_files {
                    let cfg = config.clone();
                    let tfile = track_file.clone();
                    futures.push(tokio::task::spawn_blocking(move || {
                        LocalMetadata::process_metadata(tfile, cfg)
                    }));
                }

                while let Some(handle_res) = futures.next().await {
                    match handle_res {
                        Ok(Some(metadata)) => {
                            if tx.send(metadata).await.is_err() {
                                break;
                            }
                        }
                        Err(join_err) => {
                            return Err(anyhow!("blocking task panicked: {}", join_err));
                        }
                        _ => {}
                    }
                }

                Ok::<(), anyhow::Error>(())
            });
        }

        drop(tx);

        let mut tracks = Vec::new();
        while let Some(track) = rx.recv().await {
            tracks.push(track);
        }

        Ok(tracks)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FingerprintAlgorithm {
    Chromaprint,
}

impl Default for FingerprintAlgorithm {
    fn default() -> Self {
        FingerprintAlgorithm::Chromaprint
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalMetadataConfig {
    pub cover_art_dir: PathBuf,
    pub fingerprint: FingerprintAlgorithm,
}

impl Default for LocalMetadataConfig {
    fn default() -> Self {
        LocalMetadataConfig {
            cover_art_dir: PATHS.covers_dir.clone(),
            fingerprint: FingerprintAlgorithm::default(),
        }
    }
}

fn picture_to_cover(data: &[u8], description: Option<&str>, cover_art_dir: PathBuf) -> Option<Artwork> {
    let img = ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .ok()?
        .decode()
        .ok()?;

    let mut jpeg_buf = Vec::new();
    let mut enc = JpegEncoder::new_with_quality(&mut jpeg_buf, 100);
    enc.encode_image(&img.to_rgb8()).ok()?;

    let mut hasher = Sha256::new();
    hasher.update(&jpeg_buf);
    let hash_hex = hex::encode(hasher.finalize());

    let dest = cover_art_dir.join(hash_hex).with_extension("jpg");

    if let Some(dir) = dest.parent() {
        fs::create_dir_all(dir).ok()?;
    }

    if !dest.exists() {
        fs::write(&dest, &jpeg_buf).ok()?;
    }

    Some(Artwork {
        path: dest,
        mime_type: "image/jpeg".into(),
        description: description.unwrap_or_default().into(),
    })
}
