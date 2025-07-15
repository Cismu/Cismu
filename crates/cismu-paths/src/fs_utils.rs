use fs2::FileExt;
use std::{fs, fs::OpenOptions, io, path::Path};

use tracing::{Level, instrument};

use crate::errors::Error;

/// Asegura que la carpeta `path` existe (creándola recursivamente si hace falta).
#[instrument(level = Level::TRACE, err)]
pub fn ensure_dir(path: &Path) -> Result<(), Error> {
    fs::create_dir_all(path)?;
    Ok(())
}

/// Asegura que el fichero `path` existe (creando su carpeta padre si hace falta).
#[instrument(level = Level::TRACE, err)]
pub fn ensure_file(path: &Path) -> Result<(), Error> {
    if let Some(parent) = path.parent() {
        ensure_dir(parent)?;
    }

    if !path.exists() {
        fs::File::create(path)?;
    }

    Ok(())
}

/// Adquiere un lock exclusivo sobre el fichero `path`.
/// Devuelve el `File` bloqueado; mientras conserves el handle, el lock se mantiene.
#[instrument(level = Level::TRACE, err)]
pub fn lock_file(path: &Path) -> Result<std::fs::File, Error> {
    ensure_file(path)?;
    let file = OpenOptions::new().read(true).write(true).open(path)?;
    file.lock_exclusive()?;
    Ok(file)
}

/// Verifica que `path` es escribible (tiene permisos adecuados).
#[instrument(level = Level::TRACE, err)]
pub fn check_writable(path: &Path) -> Result<(), Error> {
    let meta = fs::metadata(path)?;
    // en Unix basta con que el owner tenga permisos de escritura:
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = meta.permissions().mode();
        if mode & 0o200 == 0 {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!("No write permission for {}", path.display()),
            )
            .into());
        }
    }
    // en Windows podrías intentar abrir un fichero dummy:
    #[cfg(windows)]
    {
        if fs::OpenOptions::new().write(true).open(path).is_err() {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!("No write permission for {}", path.display()),
            )
            .into());
        }
    }
    Ok(())
}
