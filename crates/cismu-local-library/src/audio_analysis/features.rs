use std::path::PathBuf;

use anyhow::Result;

use bliss_audio::Song;
use bliss_audio::decoder::Decoder;
use bliss_audio::decoder::ffmpeg::FFmpegDecoder;

pub fn get_features(path: impl Into<PathBuf>) -> Result<Song> {
    let song = FFmpegDecoder::song_from_path(path.into())?;
    Ok(song)
}
