use std::time::Duration;

pub mod chroma;
pub mod features;

/// Datos técnicos de la grabación.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AudioDetails {
    pub duration: Duration,
    pub bitrate_kbps: Option<u32>,
    pub sample_rate_hz: Option<u32>,
    pub channels: Option<u8>,
    pub analysis: Option<AudioAnalysis>,
    pub fingerprint: Option<String>,
}

/// Análisis de la grabación.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AudioAnalysis {
    pub quality: Option<AudioQuality>,
    pub features: Option<Vec<f32>>,
    pub bpm: Option<f32>,
}

/// Calidad de la grabación.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AudioQuality {
    pub score: f32,
    pub assessment: String,
}
