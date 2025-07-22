use std::path::PathBuf;

use crate::discography::release_track::ReleaseTrackId;

use super::artist::ArtistId;
use super::genre_styles::{Genre, Style};

pub type ReleaseId = u64;

/// El Lanzamiento (Release): El producto que agrupa las pistas.
#[derive(Debug, Clone)]
pub struct Release {
    pub id: ReleaseId,
    pub title: String,
    pub format: ReleaseFormat,

    pub main_artist_ids: Vec<ArtistId>,
    pub release_tracks: Vec<ReleaseTrackId>,

    pub release_date: Option<String>,
    pub artworks: Vec<Artwork>,
    pub genres: Vec<Genre>,
    pub styles: Vec<Style>,
}

/// Define el formato principal del lanzamiento (álbum, EP, etc.).
#[derive(Debug, Clone, PartialEq)]
pub enum ReleaseFormat {
    Album,
    EP,
    Single,
    Compilation,
    Mix,
    Other,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Artwork {
    pub path: PathBuf,
    pub mime_type: String,
    pub description: Option<String>,
    pub hash: String,
    pub credits: Option<String>,
}
