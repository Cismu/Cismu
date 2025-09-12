pub mod error;
pub mod prelude;

pub mod analysis;
pub mod audio;
pub mod metadata;
pub mod pipeline;

use std::path::Path;

pub use analysis::features::FeatureSet;
pub use metadata::model::{Tag, Track};
pub use pipeline::probe::Probe;

use crate::{error::Error, pipeline::probe::ProbeResult};

/// Camino feliz: parsea metadatos y analiza audio con la config por defecto.
/// Habilita backends por `features` (p.ej. "lofty", "symphonia").
pub fn probe<P: AsRef<Path>>(path: P) -> Result<ProbeResult, Error> {
    Probe::default().run(path)
}

/// Solo metadatos (sin análisis de audio).
pub fn read_metadata<P: AsRef<Path>>(path: P) -> Result<Track, Error> {
    pipeline::probe::Probe::default().read_metadata(path)
}

/// Solo análisis (si ya tenés PCM o querés saltarte el reader por defecto).
pub fn analyze<P: AsRef<Path>>(path: P) -> Result<FeatureSet, Error> {
    pipeline::probe::Probe::default().analyze(path)
}
