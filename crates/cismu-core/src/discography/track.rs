use super::artist::ArtistId;

pub type TrackId = u64;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Track {
    pub id: TrackId,
    pub title: String,
    pub artists: Vec<ArtistId>,
}
