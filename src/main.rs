use std::path::PathBuf;

use cismu_probe::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Camino feliz
    let r = cismu_probe::probe("song.flac")?;
    println!("{:?} – features: {:?}", r.track.title, r.features);

    let probe = Probe::builder().build();

    let only_meta = probe.read_metadata("song.flac")?;
    let only_feats = probe.analyze("song.flac")?;
    let full = probe.run("song.flac")?;

    // let folder: PathBuf = "/home/undead34/Music/Soulsheek".into();
    // let files = scan_dir(&folder);
    // for file in files {
    //     println!("{}", file.display());
    // }

    Ok(())
}

pub fn scan_dir(path: &PathBuf) -> Vec<PathBuf> {
    let mut results = Vec::new();
    scan(&path, 0, 5, &mut results);
    results
}

pub fn scan(path: &PathBuf, depth: usize, max_depth: usize, results: &mut Vec<PathBuf>) {
    let valid_extensions = vec!["mp3", "flac", "wav", "opus"];
    if depth > max_depth {
        return;
    }

    if path.is_file() {
        results.push(path.to_path_buf());
        return;
    }

    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_file() {
                let extension = path.extension().and_then(|s| s.to_str());
                if valid_extensions.contains(&extension.unwrap_or_default()) {
                    results.push(path);
                }
            } else if path.is_dir() {
                scan(&path, depth + 1, max_depth, results);
            }
        }
    }
}
