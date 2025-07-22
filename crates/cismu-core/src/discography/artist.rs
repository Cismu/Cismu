pub type ArtistId = u64;

/// La entidad Artista: El creador de la música.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Artist {
    pub id: ArtistId,
    pub name: String,
    pub variations: Vec<String>,
    pub bio: Option<String>,
    pub sites: Vec<String>,
}
