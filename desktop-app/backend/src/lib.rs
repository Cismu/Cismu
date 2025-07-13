use anyhow::Result;

use tauri::async_runtime::handle;
use tauri::Manager;
use tracing::{info, instrument, Level};

// fn init_library() -> Result<LibraryManager<LocalScanner, LocalMetadata, LocalStorage>> {
//     let rt = handle();
//     let tokio_handle = rt.inner().clone();

//     let config = ConfigManager::new();
//     info!("ConfigManager created");

//     let library = LibraryManager::new(tokio_handle, config);
//     info!("LibraryManager created and ready to manage your library");

//     Ok(library)
// }

#[instrument(name = "setup_app", level = Level::DEBUG, skip(app))]
fn setup(app: &mut tauri::App) -> Result<()> {
    info!("Initializing Cismu! Are you ready?");

    // let library = init_library()?;
    // app.manage(library);
    info!("LibraryManager registered in the App");

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // This should be called as early in the execution of the app as possible
    #[cfg(debug_assertions)] // only enable instrumentation in development builds
    let devtools = tauri_plugin_devtools::init();

    let mut builder = tauri::Builder::default();

    #[cfg(debug_assertions)]
    {
        builder = builder.plugin(devtools);
    }

    builder
        .setup(|app| setup(app).map_err(Into::into))
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
