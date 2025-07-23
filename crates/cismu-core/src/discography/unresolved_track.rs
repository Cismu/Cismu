use std::{path::PathBuf, time::Duration};

use crate::discography::release::Artwork;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct UnresolvedTrack {
    // File Details
    pub path: PathBuf,
    pub file_size: u64,
    pub last_modified: u64,
    pub duration: Duration,
    pub bitrate_kbps: Option<u32>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u8>,
    // Metadata
    pub title: Option<String>,
    pub album: Option<String>,
    pub track_number: Option<u32>,
    pub disc_number: Option<u32>,
    pub genre: Option<Vec<String>>,
    pub artwork: Option<Vec<Artwork>>,

    // Credits
    pub album_artists: Vec<String>,
    pub performers: Vec<String>,
    pub featured_artists: Vec<String>,
    pub composers: Vec<String>,
    pub producers: Vec<String>,
}
