use notify::Error as NotifyError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    #[error("Configuration parse error: {0}")]
    Parse(#[from] config::ConfigError),

    #[error("Notify watcher error: {0}")]
    Notify(#[from] NotifyError),
}
