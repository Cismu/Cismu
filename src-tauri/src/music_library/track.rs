use std::{fs, path::PathBuf, time::UNIX_EPOCH};

use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use super::metadata::MIN_FILE_SIZE_BYTES;

#[derive(Debug, Clone, Serialize, Deserialize, Builder, Default)]
#[builder(setter(into, strip_option))]
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

    pub artwork: Option<Artwork>,
    pub rating: Rating,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Artwork {
    pub data: Vec<u8>,
    pub mime_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AudioInfo {
    pub duration_secs: f32,
    pub bitrate_kbps: Option<u32>,
    pub sample_rate_hz: Option<u32>,
    pub channels: Option<u8>,
    pub quality_score: Option<f32>,
    pub analysis: Option<AudioAnalysis>,
    pub tag_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AudioAnalysis {}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Rating {}
