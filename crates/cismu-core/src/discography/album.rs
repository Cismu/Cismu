use super::artist::ArtistId;
use super::genre_styles::{Genre, Style};
use super::track::Track;

pub type AlbumId = u64;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Album {
    pub id: AlbumId,
    pub title: String,
    pub artists: Vec<ArtistId>,
    pub tracks: Vec<Track>,
    pub genres: Vec<Genre>,
    pub styles: Vec<Style>,
    pub release_date: Option<String>,
    pub notes: Option<String>,
}
