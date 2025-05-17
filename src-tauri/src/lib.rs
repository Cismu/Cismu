mod music_library;

use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use music_library::{
    events::LibraryEvent, storage::JsonStorage, track::Track, LibraryConfigBuilder,
    MusicLibraryBuilder,
};

use serde::Serialize;
use tauri::ipc::Channel;

/// Stream de eventos hacia frontend
#[derive(Clone, Serialize)]
#[serde(tag = "event", content = "data", rename_all = "camelCase")]
enum ScanEvent {
    ScanStarted,
    TrackAdded(Track),
    TrackRemoved(u64),
    TrackUpdated(Track),
    ScanFinished,
    Error(String),
}

#[tauri::command]
async fn start_scan(on_event: Channel<ScanEvent>) {
    // clonamos el canal para enviarlo desde el hilo de escaneo
    let ch = on_event.clone();

    // lanzamos todo en un hilo de bloqueo
    tauri::async_runtime::spawn_blocking(move || {
        // aquí guardaremos el instante de inicio
        let start_time = Arc::new(Mutex::new(None::<Instant>));
        let start_time_cb = Arc::clone(&start_time);

        // --- setup de la librería ---
        let config = LibraryConfigBuilder::default()
            .database_path("..\\default.db")
            .scan_directories(vec!["C:\\".into(), "E:\\".into(), "D:\\".into()])
            .build()
            .unwrap();

        let storage = JsonStorage::new(config.database_path.clone());
        let mut library = MusicLibraryBuilder::new()
            .config(config)
            .storage(storage)
            .build()
            .unwrap();

        // registramos el callback de eventos
        library.on_event(move |evt| {
            // cuando empieza el escaneo, guardamos el Instant::now()
            if let LibraryEvent::ScanStarted = evt {
                let mut guard = start_time_cb.lock().unwrap();
                *guard = Some(Instant::now());
            }

            // cuando termina, calculamos y mostramos la duración
            if let LibraryEvent::ScanFinished = evt {
                if let Some(start) = *start_time_cb.lock().unwrap() {
                    let elapsed = start.elapsed();
                    println!("✔️ Escaneo completo en {:.2?}", elapsed);
                }
            }

            // reenviamos el evento al frontend
            let scan_evt = match evt {
                LibraryEvent::ScanStarted => ScanEvent::ScanStarted,
                LibraryEvent::TrackAdded(t) => ScanEvent::TrackAdded(t.clone()),
                LibraryEvent::TrackRemoved(id) => ScanEvent::TrackRemoved(*id),
                LibraryEvent::TrackUpdated(t) => ScanEvent::TrackUpdated(t.clone()),
                LibraryEvent::ScanFinished => ScanEvent::ScanFinished,
                LibraryEvent::Error(msg) => ScanEvent::Error(msg.clone()),
            };
            // en producción, capturaríamos el error de send en un log, no un unwrap
            ch.send(scan_evt).unwrap();
        });

        // arrancamos el escaneo; esto bloquea hasta que termine
        if let Err(e) = library.full_scan() {
            let _ = on_event.send(ScanEvent::Error(e.to_string()));
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![start_scan])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
