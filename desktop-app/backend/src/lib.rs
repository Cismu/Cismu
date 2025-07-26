use anyhow::Result;
use cismu_core::discography::artist::{Artist, ArtistId};
use cismu_core::discography::release::{Release, ReleaseId};
use specta_typescript::Typescript;
use tauri::{Manager, State};
use tauri_specta::Builder;

use tracing::{debug, info, instrument, Level};
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, EnvFilter};

use cismu_local_library::LibraryManager;

#[instrument(name = "init_library", level = Level::INFO, err, skip_all)]
fn init_library() -> Result<LibraryManager> {
    let _config_span = tracing::debug_span!("load_config").entered();

    let library = LibraryManager::default();
    debug!("LibraryManager instance ready");

    Ok(library)
}

#[instrument(parent = None, name = "setup_app", level = Level::INFO, err, skip(app))]
fn setup(app: &mut tauri::App) -> Result<()> {
    info!("🚀 Initializing Cismu…");

    let library = init_library()?;
    app.manage(library);
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

    let builder = Builder::<tauri::Wry>::new();

    #[cfg(debug_assertions)] // <- Only export on non-release builds
    builder
        .typ::<Release>()
        .typ::<Artist>()
        .export(Typescript::default(), "../../packages/bindings.ts")
        .expect("Failed to export typescript bindings");

    tauri::Builder::default()
        .setup(|app| setup(app).map_err(Into::into))
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            scan,
            get_all_artists,
            get_releases_for_artist,
            get_release_details
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn scan(state: State<'_, LibraryManager>) -> tauri::Result<()> {
    let library = state.inner();
    library.scan().await?;
    Ok(())
}

#[tauri::command]
fn get_all_artists(state: State<'_, LibraryManager>) -> tauri::Result<Vec<Artist>> {
    let library = state.inner();
    Ok(library.get_all_artists()?)
}

#[tauri::command]
fn get_releases_for_artist(
    state: State<'_, LibraryManager>,
    artist_id: ArtistId,
) -> tauri::Result<Vec<Release>> {
    let library = state.inner();
    Ok(library.get_releases_for_artist(artist_id)?)
}

#[tauri::command]
fn get_release_details(
    state: State<'_, LibraryManager>,
    release_id: ReleaseId,
) -> tauri::Result<Option<Release>> {
    let library = state.inner();
    Ok(library.get_release_details(release_id)?)
}
