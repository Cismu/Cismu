use std::str::FromStr;

use thiserror::Error;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Error, Debug, Clone, Copy)]
pub enum ReleaseStatusError {
    #[error("Release Status cannot be empty!")]
    InvalidInput,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(rename_all = "snake_case"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReleaseStatus {
    Official,
    Promotion,
    Bootleg,
    PseudoRelease,
    Withdrawn,
    Cancelled,
    Custom(std::string::String),
}

impl FromStr for ReleaseStatus {
    type Err = ReleaseStatusError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(ReleaseStatusError::InvalidInput);
        }

        // Misma normalización que ReleaseFormat
        let normalized: String = trimmed
            .to_lowercase()
            .chars()
            .filter(|c| !matches!(c, ' ' | '-' | '_' | '.' | '/' | '\\'))
            .collect();

        let status = match normalized.as_str() {
            // official / oficial
            "official" | "oficial" => Self::Official,

            // promotion / promo / promocional
            "promotion" | "promo" | "promotional" | "promocion" | "promocional" => Self::Promotion,

            // bootleg / unofficial / fan-made / pirate
            "bootleg" | "unofficial" | "fanmade" | "pirate" => Self::Bootleg,

            // pseudo release
            "pseudorelease" | "pseudorel" | "pseudo" => Self::PseudoRelease,

            // withdrawn / retirado(a)
            "withdrawn" | "retirado" | "retirada" => Self::Withdrawn,

            // cancelled / canceled / cancelado(a)
            "cancelled" | "canceled" | "cancelado" | "cancelada" => Self::Cancelled,

            // cualquier otra cosa → Unknown (a diferencia de ReleaseFormat que tiene Other)
            _ => Self::Custom(trimmed.to_string()),
        };

        Ok(status)
    }
}

impl std::fmt::Display for ReleaseStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Official => write!(f, "Official"),
            Self::Promotion => write!(f, "Promotion"),
            Self::Bootleg => write!(f, "Bootleg"),
            Self::PseudoRelease => write!(f, "Pseudo Release"),
            Self::Withdrawn => write!(f, "Withdrawn"),
            Self::Cancelled => write!(f, "Cancelled"),
            Self::Custom(s) => write!(f, "{}", s),
        }
    }
}
