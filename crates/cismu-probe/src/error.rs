use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O: {0}")]
    Io(#[from] std::io::Error),

    #[error("Unsupported format: {0}")]
    Unsupported(&'static str),

    #[error("Error during feature analysis")]
    Analysis(#[from] crate::analysis::features::FeatureError),

    #[cfg(feature = "ffmpeg")]
    #[error(transparent)]
    FfmpegNative(#[from] crate::audio::decoder::FFmpegNativeError),

    #[cfg(feature = "lofty")]
    #[error(transparent)]
    Lofty(#[from] crate::metadata::reader::LoftyReaderError),
}
