mod analysis;
pub mod config;
mod error;
mod events;
pub mod library;
mod metadata;
mod scanner;
pub mod storage;
mod track;
mod traits;
mod utils;

pub use config::LibraryConfigBuilder;
pub use library::MusicLibraryBuilder;
