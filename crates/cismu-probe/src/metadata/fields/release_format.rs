use std::{fmt, str::FromStr};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

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

    /// Formato desconocido o no especificado.
    Unknown,

    /// Otro formato no listado; el `String` describe cuál.
    Other(String),
}

impl Default for ReleaseFormat {
    fn default() -> Self {
        ReleaseFormat::Unknown
    }
}

impl FromStr for ReleaseFormat {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Ok(Self::Unknown);
        }

        // 2) Normalización: "Blu-ray Audio" todo termine como "blurayaudio".
        let normalized: String = trimmed
            .to_lowercase()
            .chars()
            .filter(|c| !matches!(c, ' ' | '-' | '_' | '.' | '/' | '\\'))
            .collect();

        match normalized.as_str() {
            // Digital
            "digital" | "digitaldownload" | "digitalmedia" | "download" | "web" | "file" => Ok(Self::DigitalDownload),

            // Streaming
            "streaming" | "stream" | "online" => Ok(Self::Streaming),

            // CD
            "cd" | "compactdisc" | "cdr" | "cdg" | "cdvideo" => Ok(Self::CD),

            // Vinyl
            "vinyl" | "vinilo" | "lp" | "lp12" | "lp10" | "lp7" | "picturedisc" => Ok(Self::Vinyl),

            // Cassette
            "cassette" | "tape" | "mc" => Ok(Self::Cassette),

            // Box set
            "boxset" | "box" | "cofre" => Ok(Self::BoxSet),

            // USB / memoria
            "usb" | "memory" | "memorycard" | "sdcard" | "pendrive" => Ok(Self::USB),

            // MiniDisc
            "minidisc" | "minidisk" | "minidiscmd" | "md" => Ok(Self::MiniDisc),

            // DVD (nota: mapeamos dvd-audio a DVD al no tener variante separada)
            "dvd" | "dvdaudio" | "dvda" => Ok(Self::DVD),

            // Blu-ray Audio
            "blurayaudio" | "bluray" | "bd" | "bda" | "bdaudio" | "bd-a" => Ok(Self::BluRayAudio),

            // SACD
            "sacd" | "superaudiocd" => Ok(Self::SACD),

            // Unknown explícito
            "unknown" | "desconocido" | "notspecified" | "unspecified" => Ok(Self::Unknown),

            // Cualquier otra cosa: conservar tal cual en Other
            _ => Ok(Self::Other(trimmed.to_string())),
        }
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
            Self::Unknown => write!(f, "Unknown"),
            Self::Other(value) => write!(f, "{}", value),
        }
    }
}
