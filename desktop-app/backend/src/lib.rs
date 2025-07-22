use std::sync::{Arc, Mutex};

use anyhow::Result;
use tauri::{async_runtime::handle, Manager, State};
use tracing::{debug, info, instrument, Level};

use cismu_local_library::{config_manager::ConfigManager, LibraryManager};
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, EnvFilter};

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

#[instrument(parent = None, name = "setup_app", level = Level::INFO, err, skip(app))]
fn setup(app: &mut tauri::App) -> Result<()> {
    info!("🚀 Initializing Cismu…");

    let library = init_library()?;
    app.manage(Arc::new(library));
    info!("✅ LibraryManager registered in App");

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug"));
    filter = filter
        .add_directive("off".parse().unwrap())
        .add_directive("tauri=off".parse().unwrap())
        .add_directive("cismu=debug".parse().unwrap())
        .add_directive("cismu_local_library=debug".parse().unwrap())
        .add_directive("cismu_core=debug".parse().unwrap())
        .add_directive("tauri_runtime=off".parse().unwrap());

    let fmt_layer = fmt::layer()
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true);

    tracing_subscriber::registry().with(filter).with(fmt_layer).init();

    tauri::Builder::default()
        .setup(|app| setup(app).map_err(Into::into))
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![scan])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn scan(state: State<'_, Arc<LibraryManager>>) -> tauri::Result<()> {
    let library = state.inner().clone();
    library.scan().await;

    Ok(())
}
