use std::{path::PathBuf, time::Duration};

use bliss_audio::Song;

use crate::discography::{
    album::AlbumId,
    genre_styles::{Genre, Style},
    rating::AvgRating,
};

use super::artist::ArtistId;

pub type TrackId = u64;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Track {
    pub id: TrackId,
    pub title: String,
    pub artists: Vec<ArtistId>,
    pub album: Option<AlbumId>,
    pub album_artist: Option<ArtistId>,
    pub track_number: Option<u32>,
    pub disc_number: Option<u32>,
    pub genre: Option<Vec<Genre>>,
    pub style: Option<Vec<Style>>,
    pub year: Option<String>,
    pub composer: Option<Vec<String>>,
    pub statistics: Statistics,
    pub audio_details: AudioDetails,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct UnresolvedTrack {
    pub id: TrackId,
    pub path: PathBuf,
    pub title: Option<String>,
    pub artists: Vec<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub track_number: Option<u32>,
    pub disc_number: Option<u32>,
    pub genre: Option<Vec<String>>,
    pub style: Option<Vec<String>>,
    pub year: Option<String>,
    pub composer: Option<Vec<String>>,
    pub statistics: Statistics,
    pub audio_details: AudioDetails,
}

/// Trait genérico para que cualquier tipo pueda construirse desde bliss_audio::Song
pub trait FromBlissSong<T> {
    fn from_bliss_song(song: Song) -> T;
}

/// Conversión Song  →  UnresolvedTrack
impl FromBlissSong<UnresolvedTrack> for UnresolvedTrack {
    fn from_bliss_song(song: Song) -> Self {
        UnresolvedTrack {
            path: song.path,
            title: song.title,
            artists: song.artist.into_iter().collect(),
            album: song.album,
            album_artist: song.album_artist,
            track_number: song.track_number.and_then(|n| n.try_into().ok()),
            disc_number: song.disc_number.and_then(|n| n.try_into().ok()),
            genre: song.genre.map(|g| vec![g]),
            style: None,
            year: None,
            composer: None,
            statistics: Default::default(),
            audio_details: {
                let mut details: AudioDetails = Default::default();
                details.analysis = Some(AudioAnalysis {
                    features: Some(song.analysis.as_vec()),
                    ..Default::default()
                });
                details
            },
            id: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Statistics {
    pub avg_rating: AvgRating,
    pub ratings: u32,
    pub comments: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct AudioDetails {
    pub duration: Duration,
    pub bitrate_kbps: Option<u32>,
    pub sample_rate_hz: Option<u32>,
    pub channels: Option<u8>,
    pub analysis: Option<AudioAnalysis>,
    pub fingerprint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct AudioAnalysis {
    pub quality: Option<AudioQuality>,
    pub features: Option<Vec<f32>>,
    pub bpm: Option<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AudioQuality {
    pub score: f32,
    pub assessment: String,
}
