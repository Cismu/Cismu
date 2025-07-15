use std::time::Duration;

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
