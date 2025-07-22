use crate::discography::{artist::ArtistId, rating::AvgRating};

pub type SongId = u64;

/// La Canción (Song): La obra musical abstracta.
/// Su ID (`id`) es interno, mientras que `acoustid` es para la verificación online.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Song {
    /// ID interno, único en tu base de datos.
    pub id: SongId,
    /// El ID verificado por AcoustID, `None` hasta que se confirma online.
    pub acoustid: Option<String>,
    pub title: String,

    // --- Créditos y Roles ---
    pub performer_ids: Vec<ArtistId>,
    pub featured_artist_ids: Vec<ArtistId>,
    pub composer_ids: Vec<ArtistId>,
    pub producer_ids: Vec<ArtistId>,

    // --- Estadísticas y Datos de Interacción del Usuario ---
    pub statistics: Statistics,
}

/// Estadísticas de la canción.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Statistics {
    pub avg_rating: AvgRating,
    pub ratings: u32,
    pub comments: Vec<String>,
}
