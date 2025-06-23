use bytesize::ByteSize;
use humantime_serde;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Restricciones por tipo de archivo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionConfig {
    pub min_file_size: ByteSize,
    #[serde(with = "humantime_serde")]
    pub min_duration: Duration,
}

impl ExtensionConfig {
    const COMMON_MIN_DURATION: Duration = Duration::from_secs(30);

    pub const MP3: ExtensionConfig = ExtensionConfig {
        min_file_size: ByteSize::kib(500),
        min_duration: Self::COMMON_MIN_DURATION,
    };
    pub const AAC: ExtensionConfig = ExtensionConfig {
        min_file_size: ByteSize::kib(500),
        min_duration: Self::COMMON_MIN_DURATION,
    };
    pub const MP4: ExtensionConfig = ExtensionConfig {
        min_file_size: ByteSize::mib(1),
        min_duration: Self::COMMON_MIN_DURATION,
    };
    pub const M4A: ExtensionConfig = ExtensionConfig {
        min_file_size: ByteSize::mib(1),
        min_duration: Self::COMMON_MIN_DURATION,
    };
    pub const OGG: ExtensionConfig = ExtensionConfig {
        min_file_size: ByteSize::kib(500),
        min_duration: Self::COMMON_MIN_DURATION,
    };
    pub const OPUS: ExtensionConfig = ExtensionConfig {
        min_file_size: ByteSize::kib(500),
        min_duration: Self::COMMON_MIN_DURATION,
    };
    pub const WAV: ExtensionConfig = ExtensionConfig {
        min_file_size: ByteSize::mib(5),
        min_duration: Self::COMMON_MIN_DURATION,
    };
    pub const FLAC: ExtensionConfig = ExtensionConfig {
        min_file_size: ByteSize::mib(2),
        min_duration: Self::COMMON_MIN_DURATION,
    };
}

/// Extensiones soportadas
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum SupportedExtension {
    Mp3,
    Aac,
    Mp4,
    M4a,
    Ogg,
    Opus,
    Wav,
    Flac,
}

impl SupportedExtension {
    pub const ALL: &'static [SupportedExtension] = &[
        SupportedExtension::Mp3,
        SupportedExtension::Aac,
        SupportedExtension::Mp4,
        SupportedExtension::M4a,
        SupportedExtension::Ogg,
        SupportedExtension::Opus,
        SupportedExtension::Wav,
        SupportedExtension::Flac,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            SupportedExtension::Mp3 => "mp3",
            SupportedExtension::Aac => "aac",
            SupportedExtension::Mp4 => "mp4",
            SupportedExtension::M4a => "m4a",
            SupportedExtension::Ogg => "ogg",
            SupportedExtension::Opus => "opus",
            SupportedExtension::Wav => "wav",
            SupportedExtension::Flac => "flac",
        }
    }

    pub fn config(&self) -> &'static ExtensionConfig {
        match self {
            SupportedExtension::Mp3 => &ExtensionConfig::MP3,
            SupportedExtension::Aac => &ExtensionConfig::AAC,
            SupportedExtension::Mp4 => &ExtensionConfig::MP4,
            SupportedExtension::M4a => &ExtensionConfig::M4A,
            SupportedExtension::Ogg => &ExtensionConfig::OGG,
            SupportedExtension::Opus => &ExtensionConfig::OPUS,
            SupportedExtension::Wav => &ExtensionConfig::WAV,
            SupportedExtension::Flac => &ExtensionConfig::FLAC,
        }
    }
}

impl std::str::FromStr for SupportedExtension {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower = s.to_ascii_lowercase();
        SupportedExtension::ALL
            .iter()
            .find(|ext| ext.as_str() == lower)
            .cloned()
            .ok_or_else(|| format!("Extension not supported: {}", s))
    }
}

impl std::fmt::Display for SupportedExtension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Genera el hashmap por defecto
pub fn default_extension_config() -> std::collections::HashMap<SupportedExtension, ExtensionConfig>
{
    SupportedExtension::ALL
        .iter()
        .map(|&ext| (ext, ext.config().clone()))
        .collect()
}
