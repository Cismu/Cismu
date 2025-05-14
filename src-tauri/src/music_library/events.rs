use super::utils::Track;
use anyhow::Error;

/// Eventos que puede emitir la librería
#[derive(Debug, Clone, Copy)]
pub enum LibraryEvent<'a> {
    ScanStarted,
    TrackAdded(&'a Track),
    TrackRemoved(u64),
    TrackUpdated(&'a Track),
    ScanFinished,
    Error(&'a Error),
}

/// Tipo de callback para manejar eventos
pub type EventCallback<'a> = Box<dyn FnMut(LibraryEvent<'a>) + Send + 'a>;
