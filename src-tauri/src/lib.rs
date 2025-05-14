mod music_library;

use music_library::{LibraryConfigBuilder, MusicLibraryBuilder};

#[tauri::command]
fn greet(name: &str) -> String {
    let config = LibraryConfigBuilder::default().build();
    let library = MusicLibraryBuilder::new().config(config).build();

    let mut library = match library {
        Ok(l) => l,
        Err(e) => {
            return format!("Hello, {}! We've got a problem {}!", name, e.to_string());
        }
    };

    if let Err(e) = library.full_scan() {
        return format!("Hello, {}! We've got a problem {}!", name, e.to_string());
    }

    format!("Hello, {}! Everything works as expected!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
