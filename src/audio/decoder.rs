use std::path::Path;

use crate::{
    audio::{AudioDecoder, PcmStream},
    error::Error,
};

pub struct NoopDecoder;
impl AudioDecoder for NoopDecoder {
    fn open(&self, _path: &Path) -> Result<Box<dyn PcmStream + Send>, Error> {
        Err(Error::Unsupported("audio decoder not enabled"))
    }
}

// Ejemplo con symphonia (esqueleto)
#[cfg(feature = "symphonia")]
pub struct SymphoniaDecoder;
#[cfg(feature = "symphonia")]
impl SymphoniaDecoder {
    pub fn new() -> Self {
        Self
    }
}
#[cfg(feature = "symphonia")]
impl AudioDecoder for SymphoniaDecoder {
    fn open(&self, _path: &Path) -> Result<Box<dyn PcmStream + Send>, Error> {
        // TODO: implementar
        todo!()
    }
}
