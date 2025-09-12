use anyhow::Result;
use lofty::tag::{ItemKey, Tag};
use serde::{Deserialize, Serialize};

/// Representa el estado de publicación de un lanzamiento (inspirado en MusicBrainz).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseStatus {
    Official,
    Promotion,
    Bootleg,
    PseudoRelease,
    Withdrawn,
    Cancelled,
    Unknown,
}

impl Default for ReleaseStatus {
    fn default() -> Self {
        ReleaseStatus::Unknown
    }
}

/// Conversión desde una cadena a `ReleaseStatus`, con normalización robusta.
impl From<&str> for ReleaseStatus {
    fn from(raw: &str) -> Self {
        let normalized = raw.trim().to_ascii_lowercase().replace([' ', '-', '_'], "");
        match normalized.as_str() {
            "official" => ReleaseStatus::Official,
            "promotion" | "promo" => ReleaseStatus::Promotion,
            "bootleg" | "unofficial" | "fanmade" | "pirate" => ReleaseStatus::Bootleg,
            "pseudorelease" => ReleaseStatus::PseudoRelease,
            "withdrawn" => ReleaseStatus::Withdrawn,
            "cancelled" => ReleaseStatus::Cancelled,
            _ => ReleaseStatus::Unknown,
        }
    }
}

impl std::str::FromStr for ReleaseStatus {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ReleaseStatus::from(s))
    }
}

impl From<&Tag> for ReleaseStatus {
    fn from(tag: &Tag) -> Self {
        let key = ItemKey::Unknown("RELEASESTATUS".into());

        tag.get_strings(&key)
            .map(|s| ReleaseStatus::from(s))
            .find(|st| *st != ReleaseStatus::Unknown)
            .unwrap_or_default() // Default = Unknown
    }
}
