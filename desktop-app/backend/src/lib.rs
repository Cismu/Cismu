use anyhow::Result;
use cismu_probe::{probe, read_metadata};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<()> {
    // tauri::Builder::default()
    //     .plugin(tauri_plugin_opener::init())
    //     .invoke_handler(tauri::generate_handler![])
    //     .run(tauri::generate_context!())
    //     .expect("error while running tauri application");

    let path = "/home/undead34/Music/Soulsheek/Luka Luka ★ Night Fever.flac";
    let results = read_metadata(path)?;
    println!("{:?}", results);

    Ok(())
}

// let builder = Builder::<tauri::Wry>::new();
// #[cfg(debug_assertions)] // <- Only export on non-release builds
// builder
//     .typ::<Release>()
//     .typ::<Artist>()
//     .export(Typescript::default(), "../../packages/bindings.ts")

//     .expect("Failed to export typescript bindings");
