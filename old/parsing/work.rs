/// CODIGO GENERADO POR IA NO LO HE LEDIO Y ME DA FLOJERA XD
/// LO LEERE CUANDO ALGO SE ROMPA JAJAJA

use std::collections::BTreeSet;

use lofty::tag::{ItemKey, Tag};
use unicode_normalization::UnicodeNormalization;

use crate::parsing::date::PartialDateFromTag;
use crate::work::{CreatorCredit, Language, Work, WorkKey};

const KEY_ALIAS_1: &str = "WORK_ALIASES";
const KEY_ALIAS_2: &str = "ALIASES";
const KEY_ALIAS_3: &str = "ALIAS";

const KEY_LANGUAGE_1: &str = "LANGUAGE";
const KEY_LANGUAGE_2: &str = "WORK_LANGUAGE";

const KEY_ISWC: &str = "ISWC";
const KEY_MB_WORK_ID: &str = "MUSICBRAINZ_WORKID";
const KEY_ORIGINAL_YEAR: &str = "ORIGINALYEAR";

/// Devuelve todas las cadenas para un `ItemKey` (get_strings + get_string).
fn values_for_key<'a>(tag: &'a Tag, key: &'a ItemKey) -> Vec<&'a str> {
    let mut out: Vec<&'a str> = Vec::new();
    for cow in tag.get_strings(key) {
        out.push(cow.as_ref());
    }
    if let Some(cow) = tag.get_string(key) {
        out.push(cow.as_ref());
    }
    out
}

/// Itera los valores textuales para claves Unknown(case-insensitive).
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

/// Lista (dedup) para claves Unknown separadas por `; , |`.
fn gather_unknown_list(tag: &Tag, keys: &[&str]) -> Vec<String> {
    let mut set = BTreeSet::new();
    for k in keys {
        for v in get_all_unknown(tag, k) {
            for part in v.split([';', ',', '|']) {
                let s = part.trim();
                if !s.is_empty() {
                    set.insert(s.to_string());
                }
            }
        }
    }
    set.into_iter().collect()
}

/// Primer escalar para claves Unknown.
fn first_unknown_scalar(tag: &Tag, keys: &[&str]) -> Option<String> {
    for k in keys {
        if let Some(v) = get_all_unknown(tag, k).next() {
            let s = v.trim();
            if !s.is_empty() {
                return Some(s.to_string());
            }
        }
    }
    None
}

#[inline]
fn nfkc(s: &str) -> String {
    s.nfkc().collect::<String>()
}

#[inline]
fn collapse_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[inline]
fn normalize_common_symbols(s: &str) -> String {
    s.replace(['•', '·', '–', '—', '―'], "-")
        .replace(['“', '”', '「', '」', '『', '』'], "\"")
        .replace('’', "'")
        .replace(['・', '×', '✕', '✖', '／'], "/")
}

/// Corta trailers tipo " feat. ...", " ft. ...", " featuring ...".
fn strip_feat_tail(s: &str) -> &str {
    let lower = s.to_lowercase();
    for marker in [" feat.", " ft.", " featuring "] {
        if let Some(idx) = lower.find(marker) {
            return &s[..idx];
        }
    }
    s
}

/// Quita UN bloque de paréntesis final como alias, p.ej. "8#Prince (八王子P)" -> "8#Prince".
fn strip_trailing_parens_alias(s: &str) -> &str {
    let t = s.trim();
    if t.ends_with(')') {
        if let Some(open) = t.rfind('(') {
            let inner = &t[open + 1..t.len() - 1];
            if !inner.contains([';', '/']) {
                return t[..open].trim_end();
            }
        }
    }
    t
}

fn normalize_title_for_key(raw: &str) -> String {
    let t = nfkc(raw);
    let t = normalize_common_symbols(&t);
    let t = t.to_lowercase();
    let t = t.trim();
    collapse_ws(t)
}

fn normalize_artist_for_key(raw: &str) -> String {
    let s1 = nfkc(raw);
    let s1 = normalize_common_symbols(&s1);
    let s1 = strip_feat_tail(&s1);
    let s1 = strip_trailing_parens_alias(s1);
    let s1 = s1.to_lowercase().trim().to_string();
    collapse_ws(&s1)
}

fn build_work_candidate_key(
    title: &str,
    artist_opt: Option<&str>,
    original_year: Option<u32>,
) -> WorkKey {
    // Estrategia v1b (equilibrada y offline-safe)
    let t = normalize_title_for_key(title);
    let a = artist_opt.map(normalize_artist_for_key).unwrap_or_default();
    let y = original_year.map(|y| y.to_string()).unwrap_or_default();
    WorkKey(format!("v1b|{t}|{a}|{y}"))
}

// ─────────────────────────────────────────────────────────────────────────────
// Parser principal (sin heurísticas “adivinatorias”)
// ─────────────────────────────────────────────────────────────────────────────

impl From<&Tag> for Work {
    fn from(tag: &Tag) -> Self {
        // title
        let title = values_for_key(tag, &ItemKey::TrackTitle)
            .into_iter()
            .next()
            .unwrap_or("")
            .to_string();

        // aliases (solo si vienen)
        let aliases = gather_unknown_list(tag, &[KEY_ALIAS_1, KEY_ALIAS_2, KEY_ALIAS_3]);

        // créditos (ARTISTS | ARTIST) sin roles ni artist_id
        let artist = values_for_key(tag, &ItemKey::TrackArtists)
            .into_iter()
            .next()
            .or_else(|| {
                values_for_key(tag, &ItemKey::TrackArtist)
                    .into_iter()
                    .next()
            })
            .map(|s| s.to_string());

        let mut credits = Vec::new();
        if let Some(ref name) = artist {
            credits.push(CreatorCredit {
                name: name.clone(),
                roles: Vec::new(),
                artist_id: None,
            });
        }

        // ISWC / MB Work ID si existen como unknowns
        let iswc = first_unknown_scalar(tag, &[KEY_ISWC]);
        let mbid = first_unknown_scalar(tag, &[KEY_MB_WORK_ID]);

        // LANGUAGE explícito (no inferir desde SCRIPT)
        let language: Vec<Language> = gather_unknown_list(tag, &[KEY_LANGUAGE_1, KEY_LANGUAGE_2]);

        // created: tu heurística (OriginalReleaseDate, si no ReleaseDate)
        let created = tag
            .extract_partial_date(ItemKey::OriginalReleaseDate)
            .or_else(|| tag.extract_partial_date(ItemKey::ReleaseDate));

        // original_year: solo si viene como unknown
        let original_year =
            first_unknown_scalar(tag, &[KEY_ORIGINAL_YEAR]).and_then(|s| s.parse::<u32>().ok());

        // candidate_key v1b
        let candidate_key = build_work_candidate_key(&title, artist.as_deref(), original_year);

        Work {
            id: 0, // lo asignas al persistir
            title,
            aliases,
            credits,
            iswc,
            mbid,
            language,
            created,
            candidate_key,
        }
    }
}
