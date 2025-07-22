use std::io::Cursor;
use std::path::PathBuf;

use anyhow::Result;
use thiserror::Error;

use image::{ImageReader, codecs::jpeg::JpegEncoder};
use sha2::{Digest, Sha256};

use cismu_core::discography::release_track::Artwork;
use cismu_paths::PATHS;

use tracing::error;

#[derive(Debug, Error)]
pub enum CoverError {
    #[error("ruta de destino inválida")]
    InvalidDest,
}

pub fn picture_to_cover(data: &[u8], description: Option<&str>, base_cover_dir: PathBuf) -> Result<Artwork> {
    let img = ImageReader::new(Cursor::new(data))
        .with_guessed_format()?
        .decode()?;

    let mut jpeg_buf = Vec::new();
    let mut enc = JpegEncoder::new_with_quality(&mut jpeg_buf, 100);
    enc.encode_image(&img.to_rgb8())?;

    let hash = {
        let mut hasher = Sha256::new();
        hasher.update(&jpeg_buf);
        hex::encode(hasher.finalize())
    };

    let dest = PATHS
        .cover_path(base_cover_dir, &hash, "jpg")
        .map_err(|_| CoverError::InvalidDest)?;

    if let Some(dir) = dest.parent() {
        std::fs::create_dir_all(dir)?;
    }
    if !dest.exists() {
        std::fs::write(&dest, &jpeg_buf)?;
    }

    Ok(Artwork {
        path: dest,
        mime_type: "image/jpeg".into(),
        description: description.map(str::to_string),
    })
}
