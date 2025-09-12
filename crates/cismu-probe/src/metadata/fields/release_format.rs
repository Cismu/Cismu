use std::{fmt, str::FromStr};

use thiserror::Error;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Error, Debug, Clone, Copy)]
pub enum ReleaseFormatError {
    #[error("Release Format cannot be empty!")]
    InvalidInput,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(rename_all = "snake_case"))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Formatos de lanzamiento que suelen aparecer en metadatos reales.
pub enum ReleaseFormat {
    /// Lanzamiento digital descargable (MP3, FLAC, WAV, etc.).
    DigitalDownload,

    /// Lanzamiento en plataformas de streaming (Spotify, Apple Music, etc.).
    Streaming,

    /// Disco compacto (CD estándar).
    CD,

    /// Disco de vinilo (LP, 12", 7", etc.).
    Vinyl,

    /// Casete de cinta magnética (MC).
    Cassette,

    /// Caja con varios discos/formats (Box Set).
    BoxSet,

    /// Memoria USB o tarjeta SD con el álbum.
    USB,

    /// MiniDisc (formato óptico pequeño, popular en los 90).
    MiniDisc,

    /// DVD con audio/vídeo (en lanzamientos físicos recientes).
    DVD,

    /// Blu-ray Audio (formato de alta capacidad con audio HD).
    BluRayAudio,

    /// Super Audio CD (formato de alta resolución).
    SACD,

    /// Custom formato no listado
    Custom(std::string::String),
}

impl FromStr for ReleaseFormat {
    type Err = ReleaseFormatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(ReleaseFormatError::InvalidInput);
        }

        // 2) Normalización: "Blu-ray Audio" todo termine como "blurayaudio".
        let normalized: String = trimmed
            .to_lowercase()
            .chars()
            .filter(|c| !matches!(c, ' ' | '-' | '_' | '.' | '/' | '\\'))
            .collect();

        let format = match normalized.as_str() {
            // Digital
            "digital" | "digitaldownload" | "digitalmedia" | "download" | "web" | "file" => Self::DigitalDownload,

            // Streaming
            "streaming" | "stream" | "online" => Self::Streaming,

            // CD
            "cd" | "compactdisc" | "cdr" | "cdg" | "cdvideo" => Self::CD,

            // Vinyl
            "vinyl" | "vinilo" | "lp" | "lp12" | "lp10" | "lp7" | "picturedisc" => Self::Vinyl,

            // Cassette
            "cassette" | "tape" | "mc" => Self::Cassette,

            // Box set
            "boxset" | "box" | "cofre" => Self::BoxSet,

            // USB / memoria
            "usb" | "memory" | "memorycard" | "sdcard" | "pendrive" => Self::USB,

            // MiniDisc
            "minidisc" | "minidisk" | "minidiscmd" | "md" => Self::MiniDisc,

            // DVD (nota: mapeamos dvd-audio a DVD al no tener variante separada)
            "dvd" | "dvdaudio" | "dvda" => Self::DVD,

            // Blu-ray Audio
            "blurayaudio" | "bluray" | "bd" | "bda" | "bdaudio" | "bd-a" => Self::BluRayAudio,

            // SACD
            "sacd" | "superaudiocd" => Self::SACD,

            // Cualquier otra cosa: conservar tal cual en Other
            _ => Self::Custom(trimmed.to_string()),
        };

        Ok(format)
    }
}

// Añadir esto para consistencia
impl fmt::Display for ReleaseFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DigitalDownload => write!(f, "Digital Download"),
            Self::Streaming => write!(f, "Streaming"),
            Self::CD => write!(f, "CD"),
            Self::Vinyl => write!(f, "Vinyl"),
            Self::Cassette => write!(f, "Cassette"),
            Self::BoxSet => write!(f, "Box Set"),
            Self::USB => write!(f, "USB"),
            Self::MiniDisc => write!(f, "MiniDisc"),
            Self::DVD => write!(f, "DVD"),
            Self::BluRayAudio => write!(f, "Blu-ray Audio"),
            Self::SACD => write!(f, "SACD"),
            Self::Custom(s) => write!(f, "{}", s),
        }
    }
}
