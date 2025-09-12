use crate::{error::Error, metadata::model::Track};
use std::path::Path;

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

#[cfg(feature = "lofty")]
pub struct LoftyReader {/* cfg si querés */}
#[cfg(feature = "lofty")]
impl LoftyReader {
    pub fn new() -> Self {
        Self {}
    }
}
#[cfg(feature = "lofty")]
impl MetadataReader for LoftyReader {
    fn read(&self, path: &Path, prefer_pics: bool, _ff: bool) -> Result<Track, Error> {
        // TODO: usa lofty para llenar Track
        let _prefer = prefer_pics;
        let _ = path;
        todo!()
    }
}
