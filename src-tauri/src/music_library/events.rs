use super::track::Track;

/// Eventos que emite la librería (datos siempre OWNED para no lidiar con lifetimes)
#[derive(Debug, Clone)]
pub enum LibraryEvent {
    ScanStarted,
    TrackAdded(Track),
    TrackRemoved(u64),
    TrackUpdated(Track),
    ScanFinished,
    Error(String),
}

/// Firma de un callback: recibe el evento
pub type EventCallback = Box<dyn FnMut(&LibraryEvent) + Send + 'static>;
