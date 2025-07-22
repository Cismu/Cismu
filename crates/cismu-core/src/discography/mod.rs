pub mod artist;
pub mod genre_styles;
pub mod rating;
pub mod release;
pub mod release_track;
pub mod song;
pub mod unresolved_track;

pub use unresolved_track::UnresolvedTrack;

// pub type SongId = u64;
// pub type ReleaseId = u64;
// pub type ReleaseTrackId = u64;

// /// Define el formato principal del lanzamiento.
// /// Responde a si es un Álbum, EP, Sencillo, etc.
// #[derive(Debug, Clone, PartialEq)]
// pub enum ReleaseFormat {
//     Album,
//     EP,
//     Single,
//     Compilation,
//     Mix, // Para DJ mixes, etc.
//     Other,
// }

// /// La Canción (Song): La obra musical abstracta, inmutable.
// /// Representa la grabación maestra, sin importar en qué álbum aparezca.
// #[derive(Debug, Clone, PartialEq, Default)]
// pub struct Song {
//     pub id: SongId,
//     pub title: String,

//     // Créditos y Roles

//     /// El/los intérprete(s) principal(es) de esta grabación.
//     pub performer_ids: Vec<ArtistId>,

//     /// Artistas invitados o "featuring".
//     pub featured_artist_ids: Vec<ArtistId>,

//     /// Quién(es) escribieron la canción.
//     pub composer_ids: Vec<ArtistId>,

//     /// Quién(es) produjeron la grabación.
//     pub producer_ids: Vec<ArtistId>,

//     // Datos técnicos
//     pub statistics: Statistics,
//     pub audio_details: AudioDetails,
//     pub file_details: FileDetails,
// }

// use std::{path::PathBuf, time::Duration};

// use bliss_audio::Song;

// use crate::discography::{
//     release::AlbumId,
//     genre_styles::{Genre, Style},
//     rating::AvgRating,
// };

// use super::artist::ArtistId;

// pub type TrackId = u64;

// #[derive(Debug, Clone, PartialEq, Default)]
// pub struct Track {
//     pub id: TrackId,
//     pub title: String,
//     pub artists: Vec<ArtistId>,
//     pub album: Option<AlbumId>,
//     pub album_artist: Option<ArtistId>,
//     pub track_number: Option<u32>,
//     pub disc_number: Option<u32>,
//     pub genre: Option<Vec<Genre>>,
//     pub style: Option<Vec<Style>>,
//     pub year: Option<String>,
//     pub composer: Option<Vec<String>>,
//     pub statistics: Statistics,
//     pub audio_details: AudioDetails,
// }

// #[derive(Debug, Clone, PartialEq, Default)]
// pub struct UnresolvedTrack {
//     pub id: TrackId,
//     pub title: Option<String>,
//     pub artists: Vec<String>,
//     pub album: Option<String>,
//     pub album_artist: Option<String>,
//     pub track_number: Option<u32>,
//     pub disc_number: Option<u32>,
//     pub genre: Option<Vec<String>>,
//     pub style: Option<Vec<String>>,
//     pub year: Option<String>,
//     pub composer: Option<Vec<String>>,
//     pub artwork: Option<Vec<Artwork>>,
//     pub statistics: Statistics,
//     pub audio_details: AudioDetails,
//     pub file_details: FileDetails,
// }

// /// Trait genérico para que cualquier tipo pueda construirse desde bliss_audio::Song
// pub trait FromBlissSong<T> {
//     fn from_bliss_song(song: Song) -> T;
// }

// /// Conversión Song  →  UnresolvedTrack
// impl FromBlissSong<UnresolvedTrack> for UnresolvedTrack {
//     fn from_bliss_song(song: Song) -> Self {
//         UnresolvedTrack {
//             title: song.title,
//             artists: song.artist.into_iter().collect(),
//             album: song.album,
//             album_artist: song.album_artist,
//             track_number: song.track_number.and_then(|n| n.try_into().ok()),
//             disc_number: song.disc_number.and_then(|n| n.try_into().ok()),
//             genre: song.genre.map(|g| vec![g]),
//             style: None,
//             year: None,
//             composer: None,
//             statistics: Default::default(),
//             audio_details: {
//                 let mut details: AudioDetails = Default::default();
//                 details.analysis = Some(AudioAnalysis {
//                     features: Some(song.analysis.as_vec()),
//                     ..Default::default()
//                 });
//                 details
//             },
//             id: 0,
//             file_details: FileDetails {
//                 path: song.path,
//                 ..Default::default()
//             },
//             ..Default::default()
//         }
//     }
// }

// #[derive(Debug, Clone, PartialEq, Default)]
// pub struct Statistics {
//     pub avg_rating: AvgRating,
//     pub ratings: u32,
//     pub comments: Vec<String>,
// }

// #[derive(Debug, Clone, PartialEq, Default)]
// pub struct Artwork {
//     pub path: PathBuf,
//     pub mime_type: String,
//     pub description: Option<String>,
// }

// pub type SongId = u64;
// pub type ReleaseId = u64;
// pub type ReleaseTrackId = u64;

// /// Define el formato principal del lanzamiento.
// /// Responde a si es un Álbum, EP, Sencillo, etc.
// #[derive(Debug, Clone, PartialEq)]
// pub enum ReleaseFormat {
//     Album,
//     EP,
//     Single,
//     Compilation,
//     Mix, // Para DJ mixes, etc.
//     Other,
// }

// /// La Canción (Song): La obra musical abstracta, inmutable.
// /// Representa la grabación maestra, sin importar en qué álbum aparezca.
// #[derive(Debug, Clone, PartialEq, Default)]
// pub struct Song {
//     pub id: SongId,
//     pub title: String,

//     // Créditos y Roles

//     /// El/los intérprete(s) principal(es) de esta grabación.
//     pub performer_ids: Vec<ArtistId>,

//     /// Artistas invitados o "featuring".
//     pub featured_artist_ids: Vec<ArtistId>,

//     /// Quién(es) escribieron la canción.
//     pub composer_ids: Vec<ArtistId>,

//     /// Quién(es) produjeron la grabación.
//     pub producer_ids: Vec<ArtistId>,

//     // Datos técnicos
//     pub statistics: Statistics,
//     pub audio_details: AudioDetails,
//     pub file_details: FileDetails,
// }

// /// Estadísticas de la canción.
// #[derive(Debug, Clone, PartialEq, Default)]
// pub struct Statistics {
//     pub avg_rating: AvgRating,
//     pub ratings: u32,
//     pub comments: Vec<String>,
// }

// /// Datos técnicos de la canción.
// #[derive(Debug, Clone, PartialEq, Default)]
// pub struct AudioDetails {
//     pub duration: Duration,
//     pub bitrate_kbps: Option<u32>,
//     pub sample_rate_hz: Option<u32>,
//     pub channels: Option<u8>,
//     pub analysis: Option<AudioAnalysis>,
//     pub fingerprint: Option<String>,
// }

// /// Análisis de la canción.
// #[derive(Debug, Clone, PartialEq, Default)]
// pub struct AudioAnalysis {
//     pub quality: Option<AudioQuality>,
//     pub features: Option<Vec<f32>>,
//     pub bpm: Option<f32>,
// }

// /// Calidad de la grabación.
// #[derive(Debug, Clone, PartialEq, Default)]
// pub struct AudioQuality {
//     pub score: f32,
//     pub assessment: String,
// }

// /// Información técnica del archivo.
// #[derive(Debug, Clone, PartialEq, Default)]
// pub struct FileDetails {
//     pub path: PathBuf,
//     pub size: u64,
//     pub modified: u64,
// }
