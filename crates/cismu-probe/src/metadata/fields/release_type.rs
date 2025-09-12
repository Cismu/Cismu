use std::str::FromStr;

use thiserror::Error;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Error, Debug, Clone, Copy)]
pub enum ReleaseTypeError {
    #[error("Release Type cannot be empty!")]
    InvalidInput,
}

/// Tipos de lanzamiento reconocidos (estilo Discogs/MusicBrainz).
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(rename_all = "snake_case"))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ReleaseType {
    Album,
    Single,
    EP,
    Compilation,
    Remix,
    Custom(std::string::String),
}

impl FromStr for ReleaseType {
    type Err = ReleaseTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(ReleaseTypeError::InvalidInput);
        }

        let normalized: String = s
            .trim()
            .to_ascii_lowercase()
            .chars()
            .filter(|c| !matches!(c, ' ' | '-' | '_' | '.' | '/' | '\\'))
            .collect();

        let ty = match normalized.as_str() {
            // Álbum
            "album" | "lp" | "longplay" | "fulllength" => Self::Album,

            // Single / sencillo
            "single" | "sencillo" | "onesidedsingle" | "1tracksingle" => Self::Single,

            // EP
            "ep" | "extendedplay" | "minialbum" | "minilp" => Self::EP,

            // Compilaciones
            "compilation" | "comp" | "anthology" | "bestof" | "greatesthits" | "variousartists" | "va" => {
                Self::Compilation
            }

            // Mezclas / remezclas
            "remix" | "djmix" | "mixtape" | "mix" | "continuousmix" | "mixed" => Self::Remix,

            _ => Self::Custom(trimmed.to_string()),
        };

        Ok(ty)
    }
}

impl std::fmt::Display for ReleaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Album => write!(f, "Album"),
            Self::Single => write!(f, "Single"),
            Self::EP => write!(f, "EP"),
            Self::Compilation => write!(f, "Compilation"),
            Self::Remix => write!(f, "Remix"),
            Self::Custom(s) => write!(f, "{}", s),
        }
    }
}
