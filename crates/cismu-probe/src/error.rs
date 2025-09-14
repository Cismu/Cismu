use crate::{audio::decoder::ffmpeg_native::FFmpegNativeError, prelude::FeatureError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O: {0}")]
    Io(#[from] std::io::Error),

    #[error("Unsupported format: {0}")]
    Unsupported(&'static str),

    #[error("Error during feature analysis")]
    Analysis(#[from] FeatureError),

    #[error(transparent)]
    FFmpegNative(#[from] FFmpegNativeError),
}
