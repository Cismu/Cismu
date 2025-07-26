use serde::{Deserialize, Serialize};
use std::{path::PathBuf, time::Duration};

use crate::discography::{release::ReleaseId, song::SongId};

pub type ReleaseTrackId = u64;

/// La Pista del Lanzamiento (ReleaseTrack): El "puente" entre la canción
/// abstracta y el archivo físico, conteniendo todos los datos específicos del archivo.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReleaseTrack {
    pub id: ReleaseTrackId,

    // --- Relaciones ---
    pub song_id: SongId,
    pub release_id: ReleaseId,

    // --- Metadatos de la Pista dentro del Release ---
    pub track_number: u32,
    pub disc_number: u32,
    pub title_override: Option<String>,

    // --- Datos Físicos y Técnicos del Archivo ---
    pub audio_details: AudioDetails,
    pub file_details: FileDetails,
}

/// Datos técnicos de la canción.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct AudioDetails {
    pub duration: Duration,
    pub bitrate_kbps: Option<u32>,
    pub sample_rate_hz: Option<u32>,
    pub channels: Option<u8>,
    pub analysis: Option<AudioAnalysis>,
    pub fingerprint: Option<String>,
}

/// Análisis de la canción.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct AudioAnalysis {
    pub quality: Option<AudioQuality>,
    pub features: Option<Vec<f32>>,
    pub bpm: Option<f32>,
}

/// Calidad de la grabación.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct AudioQuality {
    pub score: f32,
    pub assessment: String,
}

/// Información técnica del archivo.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct FileDetails {
    pub path: PathBuf,
    pub size: u64,
    pub modified: u64,
}
