use std::io;
use std::path::PathBuf;

use symphonia::core::codecs::CodecType;
use symphonia::core::errors::Error as SymphoniaError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MetadataError {
    #[error("The duration of the file is less than the minimum allowed.")]
    DurationTooShort,
}

#[derive(Error, Debug)]
pub enum AnalysisError {
    #[error(
        "The audio file does not have enough channels to calculate the sound quality. it is probably not playable."
    )]
    InvalidChannelNumber,

    #[error("The track has no sample rate")]
    InvalidSampleRate,

    #[error("Failed to open file: {path}")]
    FileOpen {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("Failed to probe file format")]
    ProbeFormat(#[source] SymphoniaError),

    #[error("No compatible audio track found in the file")]
    NoCompatibleTrack,

    #[error("Failed to create decoder for codec: {codec:?}")]
    CreateDecoder {
        codec: CodecType,
        #[source]
        source: SymphoniaError,
    },

    #[error("Error when generating the Hann window: wrong size {0} vs {1}")]
    HannWindowError(usize, usize),

    #[error("Failed to read audio packet")]
    PacketReadError(#[source] SymphoniaError),

    #[error("Unrecoverable decoder error")]
    DecoderError(#[source] SymphoniaError),
}
