use std::path::PathBuf;

use super::track::{FileInfo, Track, TrackBuilder};

pub const MIN_FILE_SIZE_BYTES: u64 = 1024;
pub const MIN_DURATION_SECS: f64 = 10.0;

pub fn process(track_builder: &mut TrackBuilder, path: &PathBuf) -> Option<Track> {
    match FileInfo::new(path) {
        Some(file_info) => track_builder.file(file_info),
        None => return None,
    };

    match track_builder.build() {
        Ok(track) => Some(track),
        Err(_) => None,
    }
}
