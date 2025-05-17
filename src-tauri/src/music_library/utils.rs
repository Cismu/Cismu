use std::ffi::OsStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum AudioFormat {
    Mp3,
    Aac,
    OggVorbis,
    Flac,
    Wav,
    Aiff,
    Wv,
}

impl AudioFormat {
    pub fn from_extension(extension: &OsStr) -> Option<Self> {
        match extension.to_str()?.to_lowercase().as_str() {
            "mp3" => Some(AudioFormat::Mp3),
            "aac" => Some(AudioFormat::Aac),
            "ogg" => Some(AudioFormat::OggVorbis),
            "flac" => Some(AudioFormat::Flac),
            "wav" => Some(AudioFormat::Wav),
            "aiff" | "aif" => Some(AudioFormat::Aiff),
            "wv" => Some(AudioFormat::Wv),
            _ => None,
        }
    }
}
