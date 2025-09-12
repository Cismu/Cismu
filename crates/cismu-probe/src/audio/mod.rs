pub mod decoder;

use crate::error::Error;
use std::path::Path;

pub trait PcmStream {
    /// Devuelve frames intercalados (interleaved) en f32 [-1, 1].
    fn next_chunk(&mut self) -> Result<Option<Vec<f32>>, Error>;
    /// Info opcional: sample_rate, channels, etc.
    fn format(&self) -> Option<StreamInfo> {
        None
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StreamInfo {
    pub sample_rate: u32,
    pub channels: u16,
}

pub trait AudioDecoder {
    fn open(&self, path: &Path) -> Result<Box<dyn PcmStream + Send>, Error>;
}
