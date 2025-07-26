use std::path::PathBuf;

use crate::discography::release_track::ReleaseTrackId;
use serde::{Deserialize, Serialize};
use specta::Type;

use super::artist::ArtistId;
use super::genre_styles::{Genre, Style};

pub type ReleaseId = u64;

/// El Lanzamiento (Release): El producto que agrupa las pistas.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
pub struct Release {
    #[specta(type = String)]
    pub id: ReleaseId,
    pub title: String,
    pub release_type: Vec<ReleaseType>,

    #[specta(type = String)]
    pub main_artist_ids: Vec<ArtistId>,
    #[specta(type = String)]
    pub release_tracks: Vec<ReleaseTrackId>,

    pub release_date: Option<String>,
    pub artworks: Vec<Artwork>,
    pub genres: Vec<Genre>,
    pub styles: Vec<Style>,
}

/// Define el formato principal del lanzamiento (álbum, EP, etc.).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub enum ReleaseType {
    Album,
    EP,
    Single,
    Compilation,
    Mix,
    Other,
}

impl ReleaseType {
    pub fn parse(s: &str) -> Vec<Self> {
        if s.trim().is_empty() {
            return vec![];
        }

        s.split(';')
            .map(|part| Self::from_single_str(part.trim()))
            .collect()
    }

    fn from_single_str(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "album" | "cd" | "lp" | "vinyl" | "album/cd" => ReleaseType::Album,
            "ep" => ReleaseType::EP,
            "single" => ReleaseType::Single,
            "compilation" => ReleaseType::Compilation,
            "mix" | "dj-mix" | "mixtape" => ReleaseType::Mix,
            _ => ReleaseType::Other,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Type)]
pub struct Artwork {
    pub path: PathBuf,
    pub mime_type: String,
    pub description: Option<String>,
    pub hash: String,
    pub credits: Option<String>,
}
