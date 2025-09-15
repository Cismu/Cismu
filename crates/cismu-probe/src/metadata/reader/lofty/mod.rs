mod rating;

use crate::{Track, error::Error, metadata::reader::MetadataReader};
use std::path::Path;

#[cfg(feature = "lofty")]
use lofty::{
    error::LoftyError,
    file::{AudioFile, TaggedFile, TaggedFileExt},
    probe::Probe,
    tag::{ItemKey, Tag, TagType},
};

use thiserror::Error;

#[cfg(feature = "lofty")]
#[derive(Debug, Error)]
pub enum LoftyReaderError {
    #[error(transparent)]
    Lofty(#[from] LoftyError),

    #[error("missing primary tag")]
    MissingTag,
}

#[cfg(feature = "lofty")]
pub struct LoftyReader;

#[cfg(feature = "lofty")]
impl LoftyReader {
    pub fn new() -> Self {
        Self
    }

    fn find_best_tag<'a>(&self, tagged: &'a TaggedFile) -> Option<&'a Tag> {
        tagged
            .primary_tag()
            .filter(|t| matches!(t.tag_type(), TagType::Id3v2 | TagType::Ape | TagType::VorbisComments))
            .or_else(|| tagged.tags().iter().find(|t| t.tag_type() == TagType::Id3v2))
            .or_else(|| tagged.first_tag())
    }

    pub fn get_unknown_strings(&self, tag: &Tag, key: &str) -> Vec<String> {
        use std::collections::HashSet;

        let mut matching_keys: Vec<String> = tag
            .items()
            .filter_map(|it| match it.key() {
                ItemKey::Unknown(k) if k.eq_ignore_ascii_case(key) => Some(k.clone()),
                _ => None,
            })
            .collect();

        matching_keys.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
        matching_keys.dedup_by(|a, b| a.eq_ignore_ascii_case(b));

        let mut seen_vals_ci: HashSet<String> = HashSet::new();
        let mut out: Vec<String> = Vec::new();

        for k in matching_keys {
            let ik = ItemKey::Unknown(k);

            if let Some(s) = tag.get_string(&ik) {
                let lc = s.to_lowercase();
                if seen_vals_ci.insert(lc) {
                    out.push(s.to_string()); // devolvemos el original
                }
            }

            for s in tag.get_strings(&ik) {
                let lc = s.to_lowercase();
                if seen_vals_ci.insert(lc) {
                    out.push(s.to_string());
                }
            }
        }

        out
    }

    pub fn process(&self, path: &Path, prefer_pics: bool, _fail_fast: bool) -> Result<Track, LoftyReaderError> {
        let tagged = Probe::open(path)?.read()?;
        let props = tagged.properties();
        let tag = self.find_best_tag(&tagged).ok_or(LoftyReaderError::MissingTag)?;

        let rating = rating::get_rating(self, tag);
        println!("{:?}", rating);

        Ok(Track {})
    }
}

// Implementación del trait
#[cfg(feature = "lofty")]
impl MetadataReader for LoftyReader {
    fn read(&self, path: &Path, prefer_pics: bool, fail_fast: bool) -> Result<Track, Error> {
        let t = self.process(path, prefer_pics, fail_fast)?;
        Ok(t)
    }
}
