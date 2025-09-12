#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// let builder = Builder::<tauri::Wry>::new();
// #[cfg(debug_assertions)] // <- Only export on non-release builds
// builder
//     .typ::<Release>()
//     .typ::<Artist>()
//     .export(Typescript::default(), "../../packages/bindings.ts")
//     .expect("Failed to export typescript bindings");
