use lofty::tag::{ItemKey, Tag};

/// Devuelve todos los valores asociados a una “clave libre”
/// (hoy usa `ItemKey::Unknown`, mañana se puede reimplementar)
pub fn get_unknown_strings(tag: &Tag, key: &str) -> Vec<String> {
    let mut values = Vec::new();

    if let Some(s) = tag.get_string(&ItemKey::Unknown(key.to_string())) {
        values.push(s.into());
    }

    tag.get_strings(&ItemKey::Unknown(key.to_string()))
        .map(|s| s.to_string())
        .for_each(|s| values.push(s));

    values
}
