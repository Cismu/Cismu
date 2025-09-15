use crate::metadata::fields::rating::{AvgRating, Rating, RatingValue};
use crate::metadata::reader::LoftyReader;
use lofty::id3::v2::{Frame, FrameId, Id3v2Tag};
use lofty::tag::{ItemKey, Tag, TagType};
use std::borrow::Cow;

fn clamp01(x: f32) -> f32 {
    if x.is_finite() { x.max(0.0).min(1.0) } else { 0.0 }
}

fn stars_to_rating_value(stars: f32) -> Option<RatingValue> {
    // stars: 0.0..=5.0
    RatingValue::try_new(stars)
        .ok()
        // Fallback defensivo si tu tipo espera “escalado”:
        .or_else(|| RatingValue::from_scaled_u32((clamp01(stars / 5.0) * 100.0).round() as u32))
}

fn from_popm_byte(b: u8) -> Option<RatingValue> {
    let stars = (b as f32 / 255.0) * 5.0;
    stars_to_rating_value(stars)
}

/// Intenta parsear "a@b|rating|counter" o "rating|counter" → u8 rating
fn parse_popm_like(s: &str) -> Option<u8> {
    let parts: Vec<&str> = s.split('|').collect();
    if parts.len() >= 2 {
        parts[1].trim().parse::<u8>().ok()
    } else {
        None
    }
}

/// Normaliza texto de rating a 0..5 estrellas.
fn parse_text_rating(s: &str) -> Option<RatingValue> {
    let txt = s.trim().replace(',', "."); // 4,5 → 4.5
    if txt.is_empty() {
        return None;
    }

    // 1) Formato fracción "x/y"
    if let Some((a, b)) = txt.split_once('/') {
        let (a, b) = (a.trim().parse::<f32>().ok()?, b.trim().parse::<f32>().ok()?);
        if b > 0.0 {
            let stars = clamp01(a / b) * 5.0;
            return stars_to_rating_value(stars);
        }
    }

    // 2) Porcentaje "x%"
    if let Some(num) = txt.strip_suffix('%') {
        let v = num.trim().parse::<f32>().ok()?;
        let stars = clamp01(v / 100.0) * 5.0;
        return stars_to_rating_value(stars);
    }

    // 3) Cadena estilo POPM "correo|rating|counter"
    if txt.contains('|') {
        if let Some(b) = parse_popm_like(&txt) {
            return from_popm_byte(b);
        }
    }

    // 4) Número simple
    let v = txt.parse::<f32>().ok()?;
    let stars = if (0.0..=5.0).contains(&v) {
        // ya está en estrellas
        v
    } else if (5.0..=10.0).contains(&v) {
        // probablemente 0..10
        (v / 10.0) * 5.0
    } else if (10.0..=100.0).contains(&v) {
        // porcentaje 0..100
        (v / 100.0) * 5.0
    } else if (100.0..=255.0).contains(&v) {
        // estilo POPM 0..255
        (v / 255.0) * 5.0
    } else {
        return None;
    };

    stars_to_rating_value(stars)
}

pub fn get_rating(reader: &LoftyReader, tag: &Tag) -> AvgRating {
    match tag.tag_type() {
        TagType::Id3v2 => {
            // 1) POPM (Popularimeter)
            let id3v2_tag = Id3v2Tag::from(tag.clone());
            if let Some(Frame::Popularimeter(p)) = id3v2_tag.get(&FrameId::Valid(Cow::Borrowed("POPM"))) {
                if let Some(rv) = from_popm_byte(p.rating) {
                    return AvgRating::Some {
                        mean: rv,
                        count: p.counter,
                    };
                }
            }
            AvgRating::None
        }

        TagType::VorbisComments | TagType::Ape => {
            // 2) Popularimeter como item genérico (texto/binary)
            if let Some(popm) = tag.get(&ItemKey::Popularimeter) {
                if let Some(t) = popm.value().text() {
                    // Puede ser "100" o "mail|rating|counter"
                    if let Some(rv) = parse_text_rating(t) {
                        return AvgRating::Some { mean: rv, count: 0 };
                    }
                }
                if let Some(bin) = popm.value().binary() {
                    // Si alguien guardó un único byte (0..255)
                    if bin.len() == 1 {
                        if let Some(rv) = from_popm_byte(bin[0]) {
                            return AvgRating::Some { mean: rv, count: 0 };
                        }
                    }
                }
            }

            // 3) Claves candidatas de uso común
            const CANDIDATE_KEYS: [&str; 5] = ["RATING", "RANK", "POPULARIMETER", "_CUSTOM_RATING", "FMPS_Rating"];
            let mut ratings = Vec::new();
            for key in CANDIDATE_KEYS {
                for val in reader.get_unknown_strings(tag, key) {
                    if let Some(rv) = parse_text_rating(&val) {
                        ratings.push(Rating::from(Some(rv)));
                    }
                }
            }

            if !ratings.is_empty() {
                return AvgRating::from_iter(ratings);
            }

            AvgRating::None
        }

        _ => AvgRating::None,
    }
}
