use std::{borrow::Cow, path::PathBuf, time::Duration};

use anyhow::Result;
use lofty::{
    file::{AudioFile, TaggedFileExt},
    probe::Probe,
    tag::{Accessor, ItemKey},
};

use crate::music_library::error::MetadataError;

use super::{
    analysis::get_analysis,
    track::{Artwork, AudioInfo, FileInfo, Rating, TagInfo, Track, TrackBuilder},
};

pub const MIN_FILE_SIZE_BYTES: u64 = 1024;
pub const MIN_DURATION_SECS: f64 = 10.0;

pub fn process(track_builder: &mut TrackBuilder, path: &PathBuf) -> Option<Track> {
    let file = FileInfo::new(path)?;
    track_builder.file(file);

    get_metadata(track_builder, path).ok()?;

    track_builder.build().ok()
}

fn get_metadata(track_builder: &mut TrackBuilder, path: &PathBuf) -> Result<()> {
    let mut tag_info = TagInfo::default();
    let mut audio_info = AudioInfo::default();

    let tagged_file = Probe::open(path)?.read()?;
    let properties = tagged_file.properties();

    let duration = properties.duration();
    if duration < Duration::from_secs_f64(MIN_DURATION_SECS) {
        anyhow::bail!(MetadataError::DurationTooShort);
    }

    audio_info.duration_secs = duration;
    audio_info.bitrate_kbps = properties.audio_bitrate();
    audio_info.sample_rate_hz = properties.sample_rate();
    audio_info.channels = properties.channels();

    if let (Some(sr), Some(ch)) = (audio_info.sample_rate_hz, audio_info.channels) {
        audio_info.analysis = get_analysis(path, sr, ch).ok();
    }

    let tag = tagged_file
        .primary_tag()
        .or_else(|| tagged_file.first_tag());

    if let Some(tag) = tag {
        audio_info.tag_type = Some(format!("{:?}", tag.tag_type()));

        tag_info.title = tag.title().map(Cow::into_owned);
        tag_info.artist = tag.artist().map(Cow::into_owned);
        tag_info.album = tag.album().map(Cow::into_owned);
        tag_info.album_artist = tag.get_string(&ItemKey::AlbumArtist).map(str::to_string);
        tag_info.track_number = tag.track().and_then(|n| u16::try_from(n).ok());
        tag_info.total_tracks = tag.track_total().map(|n| n as u16).or_else(|| {
            tag.get_string(&ItemKey::TrackTotal)
                .and_then(|s| s.trim().parse::<u16>().ok())
        });
        tag_info.total_discs = tag.disk_total().map(|n| n as u16).or_else(|| {
            tag.get_string(&ItemKey::DiscTotal)
                .and_then(|s| s.trim().parse::<u16>().ok())
        });
        tag_info.genre = tag.genre().map(Cow::into_owned);
        tag_info.year = tag.year();
        tag_info.composer = tag.get_string(&ItemKey::Composer).map(str::to_string);
        tag_info.publisher = tag.get_string(&ItemKey::Publisher).map(str::to_string);
        tag_info.comments = tag.comment().map(Cow::into_owned);
        tag_info.rating = Rating::from_tag(tag);

        tag_info.artwork = Some(
            tag.pictures()
                .iter()
                .map(|p| Artwork {
                    data: p.data().to_vec(),
                    mime_type: p.mime_type().map(|mt| mt.to_string()).unwrap_or_default(),
                    description: p.description().map(str::to_string).unwrap_or_default(),
                })
                .collect(),
        );
    }

    track_builder.tags(tag_info);
    track_builder.audio(audio_info);

    Ok(())
}
