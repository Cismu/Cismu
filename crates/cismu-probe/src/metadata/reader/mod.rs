#[cfg(feature = "lofty")]
pub mod lofty;

use crate::{error::Error, metadata::model::Track};
use std::path::Path;

#[cfg(feature = "lofty")]
pub use lofty::*;

pub trait MetadataReader {
    fn read(&self, path: &Path, prefer_embedded_pictures: bool, fail_fast: bool) -> Result<Track, Error>;
}

#[derive(Default)]
pub struct NoopReader;
impl MetadataReader for NoopReader {
    fn read(&self, _path: &Path, _pic: bool, _ff: bool) -> Result<Track, Error> {
        Err(Error::Unsupported("metadata reader not enabled"))
    }
}
