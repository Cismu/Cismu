use std::collections::HashSet;
use std::str::FromStr;

use lofty::tag::{ItemKey, Tag};
use serde::{Deserialize, Serialize};

const KEY_MEDIATYPE_UNKNOWN: &str = "MEDIATYPE";

/// Formatos de lanzamiento (CD, vinyl, digital, cassette, otros).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseFormat {
    CD,
    Vinyl,
    Digital,
    Cassette,
    Other(String),
    Unknown,
}

impl Default for ReleaseFormat {
    fn default() -> Self {
        ReleaseFormat::Unknown
    }
}

/// Permite hacer `"cassette".parse::<ReleaseFormat>()`.
impl FromStr for ReleaseFormat {
    type Err = ();

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let key = raw.trim().to_ascii_lowercase();
        let key = key.strip_suffix(" media").unwrap_or(&key);

        match key {
            "cd" => Ok(ReleaseFormat::CD),
            "vinyl" | "lp" => Ok(ReleaseFormat::Vinyl),
            "digital" => Ok(ReleaseFormat::Digital),
            "cassette" | "tape" => Ok(ReleaseFormat::Cassette),
            other if !other.is_empty() => Ok(ReleaseFormat::Other(other.to_string())),
            _ => Err(()),
        }
    }
}

/// Extrae todas las cadenas de texto para un ItemKey::Unknown dado (case-insensitive).
fn get_all_unknown<'a>(tag: &'a Tag, key: &'a str) -> impl Iterator<Item = &'a str> + 'a {
    tag.items().filter_map(move |item| {
        if let ItemKey::Unknown(k) = item.key() {
            if k.eq_ignore_ascii_case(key) {
                return item.value().text();
            }
        }
        None
    })
}

/// Lógica pura de decisión basada en valores extraídos.
fn decide_release_format(media_values: &[&str]) -> ReleaseFormat {
    let mut found = Vec::new();
    for raw in media_values {
        let tokens: Vec<&str> = if raw.contains('(') {
            vec![raw.split('(').next().unwrap().trim()]
        } else {
            raw.split(|c| matches!(c, ';' | '/' | ',' | '|'))
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .collect()
        };

        for tok in tokens {
            match tok.parse::<ReleaseFormat>() {
                Ok(fmt) => found.push(fmt),
                Err(_) if !tok.is_empty() => found.push(ReleaseFormat::Other(tok.to_string())),
                _ => {}
            }
        }
    }

    let mut set: HashSet<ReleaseFormat> = HashSet::new();
    for f in found {
        set.insert(f);
    }

    if set.is_empty() {
        return ReleaseFormat::Unknown;
    }

    const PRIORITY: &[ReleaseFormat] = &[
        ReleaseFormat::Cassette,
        ReleaseFormat::Vinyl,
        ReleaseFormat::CD,
        ReleaseFormat::Digital,
    ];
    for p in PRIORITY {
        if set.contains(p) {
            return p.clone();
        }
    }

    if let Some(other) = set.into_iter().find(|f| matches!(f, ReleaseFormat::Other(_))) {
        return other;
    }

    ReleaseFormat::Unknown
}

impl From<&Tag> for ReleaseFormat {
    fn from(tag: &Tag) -> Self {
        let mut media_vals: Vec<&str> = tag
            .get_strings(&ItemKey::OriginalMediaType)
            .map(|s| s.as_ref())
            .collect();

        if let Some(val) = tag.get_string(&ItemKey::OriginalMediaType) {
            media_vals.push(val.as_ref());
        }

        media_vals.extend(get_all_unknown(tag, KEY_MEDIATYPE_UNKNOWN));

        decide_release_format(&media_vals)
    }
}
