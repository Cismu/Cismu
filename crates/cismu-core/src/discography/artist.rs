use serde::{Deserialize, Serialize};
use specta::Type;

pub type ArtistId = u64;

/// La entidad Artista: El creador de la música.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Type)]
pub struct Artist {
    #[specta(type = String)]
    pub id: ArtistId,
    pub name: String,
    pub variations: Vec<String>,
    pub bio: Option<String>,
    pub sites: Vec<String>,
}
