use anyhow::Result;
use tauri::{async_runtime::handle, Manager};
use tracing::{debug, info, instrument, Level};

use cismu_local_library::{config_manager::ConfigManager, LibraryManager};

#[instrument(name = "init_library", level = Level::INFO, err, skip_all)]
fn init_library() -> Result<LibraryManager> {
    let rt = handle();
    let tokio_handle = rt.inner().clone();

    let _config_span = tracing::debug_span!("load_config").entered();
    let config = ConfigManager::new();
    debug!("ConfigManager created");

    let library = LibraryManager::new(tokio_handle, config);
    debug!("LibraryManager instance ready");

    Ok(library)
}

#[instrument(name = "setup_app", level = Level::INFO, err, skip(app))]
fn setup(app: &mut tauri::App) -> Result<()> {
    info!("🚀 Initializing Cismu…");

    let library = init_library()?;
    app.manage(library);
    info!("✅ LibraryManager registered in App");

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(debug_assertions)] // only enable instrumentation in development builds
    let devtools = tauri_plugin_devtools::init();

    let mut builder = tauri::Builder::default();

    #[cfg(debug_assertions)]
    {
        builder = builder.plugin(devtools);
    }

    // TODO: Add real logger for prod.

    builder
        .setup(|app| setup(app).map_err(Into::into))
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
