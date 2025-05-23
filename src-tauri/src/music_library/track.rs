use std::{
    borrow::Cow,
    fs,
    path::PathBuf,
    time::{Duration, UNIX_EPOCH},
};

use derive_builder::Builder;
use lofty::{
    id3::v2::{Frame, FrameId, Id3v2Tag},
    tag::{ItemKey, Tag, TagType},
};
use serde::{Deserialize, Serialize};

use super::metadata::MIN_FILE_SIZE_BYTES;

#[derive(Debug, Clone, Serialize, Deserialize, Builder, Default)]
#[builder(setter(into, strip_option), default)]
pub struct Track {
    pub id: u64,
    pub path: PathBuf,
    pub file: FileInfo,
    pub tags: TagInfo,
    pub audio: AudioInfo,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FileInfo {
    pub filename: String,
    pub size_bytes: u64,
    pub modified: u64,
}

impl FileInfo {
    pub fn new(path: &PathBuf) -> Option<FileInfo> {
        let fs_metadata = match fs::metadata(path) {
            Ok(metadata) => metadata,
            Err(_) => return None,
        };

        let file_size = fs_metadata.len();
        if file_size < MIN_FILE_SIZE_BYTES {
            return None;
        }

        let modification_time: Option<u64> = fs_metadata.modified().ok().and_then(|sys_time| {
            sys_time
                .duration_since(UNIX_EPOCH)
                .ok()
                .map(|duration| duration.as_secs())
        });

        Some(FileInfo {
            filename: path.file_name().unwrap().to_str().unwrap().to_string(),
            size_bytes: file_size,
            modified: modification_time.unwrap_or(0),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Artwork {
    pub data: Vec<u8>,
    pub mime_type: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AudioInfo {
    pub duration_secs: Duration,
    pub bitrate_kbps: Option<u32>,
    pub sample_rate_hz: Option<u32>,
    pub channels: Option<u8>,
    pub quality_score: Option<f32>,
    pub analysis: Option<AudioAnalysis>,
    pub tag_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AudioAnalysis {
    pub spectral_analysis: AnalysisOutcome,
    pub quality_score: f32,
    pub overall_assessment: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum AnalysisOutcome {
    /// A significant drop was detected, indicating a cutoff.
    CutoffDetected {
        /// The starting frequency (Hz) of the band where the drop was first detected.
        cutoff_frequency_hz: f32,
        /// The calculated average dB level in the reference band.
        reference_level_db: f32,
        /// The average dB level in the band where the cutoff was detected.
        cutoff_band_level_db: f32,
    },
    /// No significant drop was detected within the analyzed frequency range.
    NoCutoffDetected {
        /// The calculated average dB level in the reference band.
        reference_level_db: f32,
        /// The highest frequency (Hz) analyzed.
        max_analyzed_freq_hz: f32,
    },
    /// Analysis could not be performed reliably due to insufficient audio data.
    InconclusiveNotEnoughWindows {
        /// Number of windows processed.
        processed_windows: usize,
        /// Minimum number of windows required for analysis.
        required_windows: usize,
    },
    /// Analysis failed because the reference dB level could not be calculated.
    /// This might happen if the reference frequency range is outside the spectrum data.
    InconclusiveReferenceBandError,
    /// Analysis is considered unreliable because the signal level in the reference band is too low.
    InconclusiveLowReferenceLevel {
        /// The calculated average dB level in the reference band.
        reference_level_db: f32,
    },

    InconclusiveError,
}

impl Default for AnalysisOutcome {
    fn default() -> Self {
        AnalysisOutcome::InconclusiveError
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq)]
pub enum Rating {
    Unrated,
    Stars(f32),
}

impl Rating {
    pub fn new(value: f32) -> Self {
        if value < 1.0 || value > 5.0 {
            Rating::Unrated
        } else {
            let v = (value * 100.0).round() / 100.0;
            Rating::Stars(v)
        }
    }

    pub fn is_unrated(&self) -> bool {
        matches!(self, Rating::Unrated)
    }

    pub fn stars(&self) -> Option<f32> {
        match self {
            Rating::Stars(v) => Some(*v),
            Rating::Unrated => None,
        }
    }

    pub fn from_tag(tag: &Tag) -> Self {
        match tag.tag_type() {
            TagType::Id3v2 => {
                let id3v2: Id3v2Tag = Id3v2Tag::from(tag.clone());

                if let Some(Frame::Popularimeter(popm)) =
                    id3v2.get(&FrameId::Valid(Cow::Borrowed("POPM")))
                {
                    return Self::from_popm_score(popm.rating);
                }

                Self::Unrated
            }
            _ => {
                if let Some(item) = tag.get(&ItemKey::Popularimeter) {
                    let value = item.value();

                    if let Some(bytes) = value.binary() {
                        if let Some(&raw) = bytes.last() {
                            if (1..=100).contains(&raw) {
                                return Self::from_vorbis_str(&raw.to_string());
                            }
                        }
                    } else if let Some(txt) = value.text() {
                        return Self::from_vorbis_str(txt);
                    }
                }

                if let Some(txt) = tag.get_string(&ItemKey::Unknown("RATING".into())) {
                    return Self::from_vorbis_str(txt);
                }

                Self::Unrated
            }
        }
    }
}

// ==============================
// Vorbis
// ==============================
impl Rating {
    pub fn from_vorbis_str(s: &str) -> Self {
        match s.trim().parse::<u8>() {
            Ok(n) if (1..=100).contains(&n) => {
                let stars = (n as f32 / 20.0 * 100.0).round() / 100.0;
                Rating::Stars(stars)
            }
            _ => Rating::Unrated,
        }
    }

    pub fn from_vorbis_score(score: u8) -> Self {
        Self::from_vorbis_str(&score.to_string())
    }

    pub fn to_vorbis_score(self) -> u8 {
        match self {
            Rating::Stars(v) => (v * 20.0).round().clamp(1.0, 100.0) as u8,
            Rating::Unrated => 0,
        }
    }

    pub fn to_vorbis_string(self) -> String {
        self.to_vorbis_score().to_string()
    }
}

// ==============================
// ID3v2 / POPM
// ==============================
impl Rating {
    pub fn from_popm_score(score: u8) -> Self {
        if score == 0 {
            Rating::Unrated
        } else {
            let v = 1.0 + (score as f32 / 255.0 * 4.0);
            let rounded = (v * 100.0).round() / 100.0;
            Rating::Stars(rounded)
        }
    }

    pub fn from_popm_str(s: &str) -> Self {
        match s.trim().parse::<u8>() {
            Ok(byte) => Self::from_popm_score(byte),
            Err(_) => Rating::Unrated,
        }
    }

    pub fn to_popm_score(self) -> u8 {
        match self {
            Rating::Stars(v) => ((v - 1.0) / 4.0 * 255.0).round().clamp(0.0, 255.0) as u8,
            Rating::Unrated => 0,
        }
    }

    pub fn to_popm_string(self) -> String {
        self.to_popm_score().to_string()
    }
}

impl Default for Rating {
    fn default() -> Self {
        Rating::Unrated
    }
}

impl std::fmt::Display for Rating {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Rating::Stars(v) => write!(f, "★ {:.2}", v),
            Rating::Unrated => write!(f, "Unrated"),
        }
    }
}
