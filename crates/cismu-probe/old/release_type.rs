use std::collections::HashSet;
use std::str::FromStr;

use lofty::tag::{ItemKey, Tag};
use serde::{Deserialize, Serialize};

const KEY_MEDIATYPE: &str = "MEDIATYPE";
const KEY_RELEASE_TYPE1: &str = "RELEASETYPE";
const KEY_RELEASE_TYPE2: &str = "MUSICBRAINZ_RELEASE_GROUP_TYPE";

/// Tipos de lanzamiento reconocidos.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseType {
    Album,
    Single,
    EP,
    Compilation,
    Remix,
    Unknown,
}

impl Default for ReleaseType {
    fn default() -> Self {
        ReleaseType::Unknown
    }
}

/// Permite hacer `"album".parse::<ReleaseType>()`.
impl FromStr for ReleaseType {
    type Err = ();

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let key = raw.trim().to_ascii_lowercase().replace([' ', '-', '_'], "");

        match key.as_str() {
            "album" | "lp" | "longplay" => Ok(ReleaseType::Album),
            "single" | "sencillo" => Ok(ReleaseType::Single),
            "ep" | "extendedplay" => Ok(ReleaseType::EP),
            "compilation" | "comp" | "anthology" | "bestof" | "greatesthits" => Ok(ReleaseType::Compilation),
            "remix" | "djmix" | "mixtape" | "set" | "mix" | "continuousmix" => Ok(ReleaseType::Remix),
            _ => Err(()),
        }
    }
}

/// Input intermedio, desacoplado del parser/tagger.
#[derive(Debug)]
struct ReleaseTypeInput<'a> {
    is_compilation_flag: bool,
    media_types: Vec<&'a str>,
    release_types: Vec<&'a str>,
}

/// Lógica pura de decisión basada en ReleaseTypeInput.
fn decide_release_type(input: &ReleaseTypeInput) -> ReleaseType {
    // 1) Flag de compilación
    if input.is_compilation_flag {
        return ReleaseType::Compilation;
    }

    // 2) Buscar token dentro de paréntesis en MEDIATYPE
    for m in &input.media_types {
        if let (Some(start), Some(end)) = (m.find('('), m.rfind(')')) {
            if let Ok(rt) = m[start + 1..end].parse::<ReleaseType>() {
                return rt;
            }
        }
    }

    // 3) Parsear todos los release_types y recogerlos en un HashSet
    let mut found = HashSet::new();
    for val in &input.release_types {
        for token in val.split(|c| matches!(c, ';' | '/' | ',' | '|')) {
            if let Ok(rt) = token.parse::<ReleaseType>() {
                found.insert(rt);
            }
        }
    }

    // 4) Priorizar según orden definido
    const PRIORITY: &[ReleaseType] = &[
        ReleaseType::Remix,       // lo más distinto: remezclas
        ReleaseType::Compilation, // colecciones
        ReleaseType::Single,      // solo 1-2 pistas
        ReleaseType::EP,          // 3-6 pistas
        ReleaseType::Album,       // álbum completo
    ];

    for &p in PRIORITY {
        if found.contains(&p) {
            return p;
        }
    }

    ReleaseType::Unknown
}

/// Extrae todas las cadenas de texto asociadas a una clave (case-insensitive).
fn get_all_strings<'a>(tag: &'a Tag, key: &'a str) -> impl Iterator<Item = &'a str> + 'a {
    tag.items().filter_map(move |item| {
        if let ItemKey::Unknown(k) = item.key() {
            if k.eq_ignore_ascii_case(key) {
                return item.value().text();
            }
        }
        None
    })
}

/// Integración con `lofty`: extrae de un `Tag` el `ReleaseType`.
impl From<&Tag> for ReleaseType {
    fn from(tag: &Tag) -> Self {
        // ¿Flag de compilación?
        let is_comp = tag
            .get(&ItemKey::FlagCompilation)
            .and_then(|item| item.value().text())
            .map_or(false, |s| matches!(s.trim(), "1" | "true" | "yes"));

        // Valores de MEDIATYPE
        let media_types: Vec<_> = get_all_strings(tag, KEY_MEDIATYPE).collect();

        // Valores de RELEASETYPE y MUSICBRAINZ_RELEASE_GROUP_TYPE
        let release_types: Vec<_> = [KEY_RELEASE_TYPE1, KEY_RELEASE_TYPE2]
            .iter()
            .flat_map(|&k| get_all_strings(tag, k))
            .collect();

        let input = ReleaseTypeInput {
            is_compilation_flag: is_comp,
            media_types,
            release_types,
        };

        decide_release_type(&input)
    }
}
