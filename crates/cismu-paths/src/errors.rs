use std::io;

/// Errores genéricos del crate
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// No se pudo determinar el directorio base (HOME, XDG, etc)
    #[error(
        "Could not determine the project directory, the call to ProjectDirs failed, \
         the system probably does not provide a valid $HOME path."
    )]
    NoHome,

    /// Hash inválido para cover: no es hex o demasiado corto
    #[error("Invalid cover hash: {0}. Must be ≥2 hex characters.")]
    InvalidCoverHash(String),

    /// Error de IO al crear dirs, ficheros, locks...
    #[error(transparent)]
    Io(#[from] io::Error),
}
