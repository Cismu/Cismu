use super::genre_styles::Genre;

pub type ArtistId = u64;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Artist {
    pub id: ArtistId,
    pub name: String,
    pub variations: Vec<String>,
    pub bio: Option<String>,
    pub sites: Vec<String>,
    pub genres: Vec<Genre>,
}
