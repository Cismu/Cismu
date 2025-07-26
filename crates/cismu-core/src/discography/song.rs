use crate::discography::{artist::ArtistId, rating::AvgRating};
use serde::{Deserialize, Serialize};

pub type SongId = u64;

/// La Canción (Song): La obra musical abstracta.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Song {
    /// Identificador único de la canción dentro del sistema.
    pub id: SongId,
    /// La "huella digital" acústica de la canción, para verificación online.
    pub acoustid: Option<String>,
    /// El título de la canción.
    pub title: String,
    /// El/los intérprete(s) principal(es) de la canción.
    pub performer_ids: Vec<ArtistId>,
    /// El/los artista(s) invitado(s) o colaborador(es).
    pub featured_ids: Vec<ArtistId>,
    /// El/los compositor(es) de la letra y/o música.
    pub composer_ids: Vec<ArtistId>,
    /// El/los productor(es) que supervisaron la grabación.
    pub producer_ids: Vec<ArtistId>,

    /// Estadísticas de interacción del usuario con la canción.
    pub statistics: Statistics,
}

/// Contiene las estadísticas de interacción de una canción.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Statistics {
    /// La calificación promedio, calculada a partir de todas las valoraciones.
    pub avg_rating: AvgRating,
    /// El número total de valoraciones que ha recibido la canción.
    pub ratings: u32,
    /// Una lista de comentarios dejados por los usuarios.
    pub comments: Vec<String>,
}

// _custom_rating,title,ARTIST
// let artist_id_samfree: ArtistId = 55;

// let song = Song {
//     id: 12345,
//     acoustid: None,
//     title: "ルカルカ★ナイトフィーバー⁣ / Luka Luka★Night Fever".to_string(),
//     performer_ids: vec![artist_id_samfree],
//     featured_ids: vec![],
//     composer_ids: vec![],
//     producer_ids: vec![],
//     statistics: Statistics {
//         avg_rating: AvgRating(10.0),
//         ratings: 1,
//         comments: vec![],
//     },
// };

// println!("{:#?}", song);
