use anyhow::Error;

use super::track::Track;

/// Eventos que puede emitir la librería
#[derive(Debug, Clone)]
pub enum LibraryEvent<'a> {
    ScanStarted,
    TrackAdded(Track),
    TrackRemoved(u64),
    TrackUpdated(&'a Track),
    ScanFinished,
    Error(String),
}

/// Tipo de callback para manejar eventos
pub type EventCallback<'a> = Box<dyn FnMut(LibraryEvent<'a>) + Send + 'a>;
