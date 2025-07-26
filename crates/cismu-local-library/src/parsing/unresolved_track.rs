use std::{path::PathBuf, time::Duration};

use cismu_core::discography::release::Artwork;

/// Representa una pista de audio escaneada del sistema de archivos,
/// con todos sus metadatos extraídos pero aún sin "resolver"
/// (es decir, sin enlazar a IDs de la base de datos).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct UnresolvedTrack {
    // --- Detalles Físicos del Archivo ---
    /// La ruta completa al archivo de audio.
    pub path: PathBuf,
    /// El tamaño del archivo en bytes.
    pub file_size: u64,
    /// La fecha de última modificación del archivo (timestamp Unix).
    pub last_modified: u64,

    // --- Propiedades del Audio ---
    /// La duración total de la pista.
    pub duration: Duration,
    /// La tasa de bits en kilobits por segundo (kbps).
    pub bitrate_kbps: Option<u32>,
    /// La frecuencia de muestreo en Hercios (Hz).
    pub sample_rate: Option<u32>,
    /// El número de canales de audio (ej. 1 para mono, 2 para estéreo).
    pub channels: Option<u8>,

    // --- Metadatos de la Pista (Track) ---
    /// El título de la pista individual.
    pub track_title: Option<String>,
    /// El número de pista dentro de su disco.
    pub track_number: Option<u32>,
    /// El número del disco en un lanzamiento de varios discos.
    pub disc_number: Option<u32>,
    /// Lista de géneros asociados a la pista.
    pub genres: Option<Vec<String>>,

    // --- Metadatos del Lanzamiento (Release) ---
    /// El título del lanzamiento (álbum, single, EP).
    pub release_title: Option<String>,
    /// El tipo de lanzamiento (ej. "album", "compilation", "single").
    pub release_type: Option<String>,
    /// La fecha de lanzamiento, idealmente en formato YYYY-MM-DD.
    pub release_date: Option<String>,
    /// El sello o casa discográfica.
    pub record_label: Option<String>,
    /// El número de catálogo del lanzamiento.
    pub catalog_number: Option<String>,
    /// Lista de artes de portada extraídas del archivo.
    pub artworks: Option<Vec<Artwork>>,

    // --- Créditos (como Strings sin resolver) ---
    /// Los artistas principales a nivel de lanzamiento (ej. "Various Artists").
    pub release_artists: Vec<String>,
    /// Los intérpretes principales de esta pista.
    pub track_performers: Vec<String>,
    /// Los artistas invitados o colaboradores en esta pista.
    pub track_featured: Vec<String>,
    /// Los compositores de la música/letra de esta pista.
    pub track_composers: Vec<String>,
    /// Los productores que supervisaron la grabación de esta pista.
    pub track_producers: Vec<String>,
}
