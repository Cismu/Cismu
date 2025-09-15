use cismu_probe::audio_features::{AudioDetails, FileDetails};
use cismu_probe::parsing::date::PartialDateFromTag;
use cismu_probe::{Country, ReleaseFormat, ReleaseStatus, ReleaseType};

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, UNIX_EPOCH};

use anyhow::Result;
use lofty::tag::ItemKey;
use lofty::{
    file::{AudioFile, TaggedFile, TaggedFileExt},
    probe::Probe,
    tag::{Tag, TagType},
};

use cismu_probe::{AvgRating, parsing::genre_and_style::get_genre_and_style};

pub fn scan(path: &Path, depth: usize, max_depth: usize, results: &mut Vec<PathBuf>) {
    if depth > max_depth {
        return;
    }

    if path.is_file() {
        results.push(path.to_path_buf());
        return;
    }

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_file() {
                results.push(path);
            } else if path.is_dir() {
                scan(&path, depth + 1, max_depth, results);
            }
        }
    }
}

pub fn scan_dir(path: &str) -> Vec<PathBuf> {
    let mut results = Vec::new();
    scan(Path::new(path), 0, 5, &mut results);
    results
}

fn find_best_tag(tagged: &TaggedFile) -> Option<&Tag> {
    // 1) Si primary_tag existe y es uno de los tipos preferidos, devuélvelo.
    tagged
        .primary_tag()
        .filter(|t| matches!(t.tag_type(), TagType::Id3v2 | TagType::Ape | TagType::VorbisComments))
        // 2) Si no, busca el primer Id3v2 en todos los tags.
        .or_else(|| tagged.tags().iter().find(|t| t.tag_type() == TagType::Id3v2))
        // 3) Si no hay Id3v2, devuelve el primer tag disponible.
        .or_else(|| tagged.first_tag())
}

fn get_metadata(path: impl Into<PathBuf>) -> Result<()> {
    let path = path.into();
    let tagged = Probe::open(&path)?.read()?;
    let props = tagged.properties();
    let tag = find_best_tag(&tagged);

    if let Some(tag) = tag {
        let rating = AvgRating::from(tag);
        let (genre, style) = get_genre_and_style(tag);
        let release_status = ReleaseStatus::from(tag);

        // let release_type = ReleaseType::from(tag);
        // let release_format = ReleaseFormat::from(tag);
        // let country = Country::try_from(tag).ok();
        // let recording_date = tag.extract_partial_date(ItemKey::RecordingDate);
        // let release_date = tag
        //     .extract_partial_date(ItemKey::OriginalReleaseDate)
        //     .or_else(|| tag.extract_partial_date(ItemKey::ReleaseDate));

        // if let Ok(metadata) = std::fs::metadata(&path) {
        //     let modified = metadata
        //         .modified()
        //         .ok()
        //         .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        //         .map(|d| d.as_secs())
        //         .ok_or(anyhow::anyhow!("Failed to get modified time"))?;

        //     let file_details = FileDetails {
        //         path: path.clone(),
        //         size: metadata.len(),
        //         modified,
        //     };

        //     // println!("File Details: {:#?}", file_details);
        // }

        // let audio_details = AudioDetails {
        //     duration: props.duration(),
        //     bitrate_kbps: props.audio_bitrate(),
        //     sample_rate_hz: props.sample_rate(),
        //     channels: props.channels(),
        //     analysis: None,
        //     fingerprint: None,
        // };

        println!("{}", path.display());
        println!("Genre: {:?}", genre);
        println!("Style: {:?}", style);
        println!("Rating: {}", rating);
        println!("Release Status: {:?}", release_status);
        // println!("Release Type: {:?}", release_type);
        // println!("Release Format: {:?}", release_format);
        // println!("Country: {:?}", country);
        // println!("Recording Date: {:?}", recording_date);
        // println!("Release Date: {:?}", release_date);
        // println!("Audio Details: {:#?}", audio_details);
    }

    Ok(())
}

fn main() -> Result<()> {
    let scan_path = "/home/undead34/Music/Soulsheek";
    let paths = scan_dir(scan_path);

    for path in paths {
        get_metadata(&path).ok();
    }

    Ok(())
}
