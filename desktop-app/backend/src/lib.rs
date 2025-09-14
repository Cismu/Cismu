use anyhow::Result;
use cismu_probe::Probe;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<()> {
    // tauri::Builder::default()
    //     .plugin(tauri_plugin_opener::init())
    //     .invoke_handler(tauri::generate_handler![])
    //     .run(tauri::generate_context!())
    //     .expect("error while running tauri application");

    let p = Probe::builder().build();
    let path = "/home/undead34/Music/Soulsheek/[2015] Is It Wrong to Try to Pick Up Girls in a Dungeon [Single] Hey World [1000564509] [FLAC]/01 Hey World.flac";
    let f = p.analyze(path)?;
    println!("{:?}", f);

    Ok(())
}

// let builder = Builder::<tauri::Wry>::new();
// #[cfg(debug_assertions)] // <- Only export on non-release builds
// builder
//     .typ::<Release>()
//     .typ::<Artist>()
//     .export(Typescript::default(), "../../packages/bindings.ts")

//     .expect("Failed to export typescript bindings");
