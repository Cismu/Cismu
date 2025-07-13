use std::{borrow::Cow, path::PathBuf, time::Duration};

use derive_builder::Builder;
use lofty::{
    id3::v2::{Frame, FrameId, Id3v2Tag},
    tag::{ItemKey, Tag, TagType},
};

use crate::scanner::TrackFile;

#[derive(Debug, Clone, PartialEq, Default, Builder)]
#[builder(default)]
pub struct TrackMetadata {
    pub id: u64,
    pub file: FileInfo,
    pub tags: TagInfo,
    pub audio: AudioInfo,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct FileInfo {
    pub path: PathBuf,
    pub filename: String,
    pub size_bytes: u64,
    pub modified: u64,
}

impl FileInfo {
    pub fn new(file: TrackFile) -> Self {
        let path = file.path;

        FileInfo {
            path: path.clone(),
            filename: path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned(),
            size_bytes: file.file_size,
            modified: file.last_modified,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct TagInfo {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,

    pub track_number: Option<u16>,
    pub total_tracks: Option<u16>,
    pub disc_number: Option<u16>,
    pub total_discs: Option<u16>,

    pub genre: Option<String>,
    pub year: Option<u32>,
    pub composer: Option<String>,
    pub publisher: Option<String>,
    pub comments: Option<String>,

    pub artwork: Option<Vec<Artwork>>,
    pub rating: Rating,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Artwork {
    pub path: PathBuf,
    pub mime_type: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Rating {
    Unrated,
    Stars(f32),
}

impl Default for Rating {
    fn default() -> Self {
        Rating::Unrated
    }
}

impl Rating {
    pub fn from_tag(tag: &Tag) -> Self {
        match tag.tag_type() {
            TagType::Id3v2 => {
                let id3v2: Id3v2Tag = Id3v2Tag::from(tag.clone());
                if let Some(Frame::Popularimeter(popm)) =
                    id3v2.get(&FrameId::Valid(Cow::Borrowed("POPM")))
                {
                    return Rating::from_popm_score(popm.rating);
                }
                Rating::Unrated
            }
            _ => {
                if let Some(item) = tag.get(&ItemKey::Popularimeter) {
                    if let Some(bytes) = item.value().binary() {
                        if let Some(&raw) = bytes.last() {
                            return Rating::from_vorbis_score(raw);
                        }
                    } else if let Some(txt) = item.value().text() {
                        return Rating::from_vorbis_str(txt);
                    }
                }
                Rating::Unrated
            }
        }
    }

    fn from_vorbis_str(s: &str) -> Self {
        match s.trim().parse::<u8>() {
            Ok(n) if (1..=100).contains(&n) => Rating::Stars((n as f32 / 20.0 * 100.0).round() / 100.0),
            _ => Rating::Unrated,
        }
    }

    fn from_vorbis_score(score: u8) -> Self {
        Rating::from_vorbis_str(&score.to_string())
    }

    fn from_popm_score(score: u8) -> Self {
        if score == 0 {
            Rating::Unrated
        } else {
            let v = 1.0 + (score as f32 / 255.0 * 4.0);
            Rating::Stars((v * 100.0).round() / 100.0)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct AudioInfo {
    pub duration_secs: Duration,
    pub bitrate_kbps: Option<u32>,
    pub sample_rate_hz: Option<u32>,
    pub channels: Option<u8>,
    pub analysis: Option<AudioAnalysis>,
    pub tag_type: Option<String>,
    pub fingerprint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum AnalysisOutcome {
    CutoffDetected {
        cutoff_frequency_hz: f32,
        reference_level_db: f32,
        cutoff_band_level_db: f32,
    },
    NoCutoffDetected {
        reference_level_db: f32,
        max_analyzed_freq_hz: f32,
    },
    InconclusiveNotEnoughWindows {
        processed_windows: usize,
        required_windows: usize,
    },
    InconclusiveReferenceBandError,
    InconclusiveLowReferenceLevel {
        reference_level_db: f32,
    },
    InconclusiveError,
}

impl Default for AnalysisOutcome {
    fn default() -> Self {
        AnalysisOutcome::InconclusiveError
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct AudioAnalysis {
    pub spectral_analysis: AnalysisOutcome,
    pub quality_score: f32,
    pub overall_assessment: String,
}
