use std::path::Path;

use bliss_audio::decoder::Decoder as DecoderTrait;
use bliss_audio::decoder::ffmpeg::FFmpegDecoder as Decoder;
use bliss_audio::{Analysis as BlissAnalysis, BlissError, FeaturesVersion};
use thiserror::Error;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::analysis::chroma::fingerprint_from_file;
use crate::analysis::quality::{self, QualityError, QualityReport};
use crate::audio::PcmStream;

#[derive(Error, Debug, Clone)]
pub enum FeatureError {
    #[error("Audio library error")]
    Audio(#[from] BlissError),

    #[error("An error occurred while analyzing the characteristics of the file {path}.")]
    Analysis { path: String, source: BlissError },

    #[error("An error occurred while analyzing the audio quality.")]
    Quality(#[from] QualityError),
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct FeatureFlags: u32 {
        const BLISS_AUDIO       = 1 << 0;
        const CHROMAPRINT       = 1 << 1;
        const AUDIO_QUALITY     = 1 << 2;
        const ALL = Self::BLISS_AUDIO.bits()
                  | Self::AUDIO_QUALITY.bits()
                  | Self::CHROMAPRINT.bits();
    }
}

impl FeatureFlags {
    pub fn default_all() -> Self {
        FeatureFlags::ALL
    }
}

#[derive(Debug, Clone, Default)]
pub struct BlissFeatures {
    pub version: FeaturesVersion,
    pub analysis: BlissAnalysis,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default, Debug, Clone)]
pub struct Analysis {
    bliss: Option<BlissFeatures>,
    fingerprint: Option<String>,
    quality: Option<QualityReport>,
}

pub fn compute<P: AsRef<Path>>(
    stream: &mut (dyn PcmStream + Send),
    path: P,
    features_flags: FeatureFlags,
) -> Result<Analysis, FeatureError> {
    let mut analysis = Analysis::default();

    if features_flags.contains(FeatureFlags::BLISS_AUDIO) {
        println!("Calculando características de Bliss Audio...");
        let song = Decoder::song_from_path(path.as_ref()).map_err(|err| FeatureError::Analysis {
            path: path.as_ref().display().to_string(),
            source: err,
        })?;

        analysis.bliss = Some(BlissFeatures {
            version: song.features_version,
            analysis: song.analysis,
        })
    }

    if features_flags.contains(FeatureFlags::CHROMAPRINT) {
        println!("Calculando Chromaprint...");
        analysis.fingerprint = fingerprint_from_file(path.as_ref()).ok();
    }

    if features_flags.contains(FeatureFlags::AUDIO_QUALITY) {
        println!("Calculando calidad de audio...");
        analysis.quality = quality::analyze_stream(stream).ok();
    }

    Ok(analysis)
}
