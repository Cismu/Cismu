use crate::style::Style;
use crate::{genre::Genre, utils::get_unknown_strings};

use std::borrow::Cow;
use std::collections::HashSet;
use std::str::FromStr;

use lofty::tag::{Accessor, ItemKey, Tag};
use unicode_normalization::UnicodeNormalization;

/// Separadores habituales en campos "Genre/Style".
const SEP_CHARS: &[char] = &[';', ',', '/', '\\', '|', '·', '•', '・', '／', '、', '；', '，', '\0'];

/// Normalización para facilitar el split:
/// - NFKC para homógrafos y signos "fullwidth"
/// - Colapsa whitespace Unicode a un espacio
/// - Reemplaza " espacio + guion (−–—) + espacio " por coma
fn normalize_for_splitting(input: &str) -> String {
    // 1) NFKC
    let s: String = input.nfkc().collect();

    // 2) Colapsa cualquier whitespace Unicode a ' '
    let s = s.split_whitespace().collect::<Vec<_>>().join(" ");

    // 3) Sustituye “ S + dash + S ” por coma
    s.replace(" - ", ",").replace(" – ", ",").replace(" — ", ",")
}

/// Divide un campo de tag (posiblemente con múltiples géneros/estilos) en piezas limpias.
fn split_tag_field(s: &str) -> Vec<String> {
    let pre = normalize_for_splitting(s);
    pre.split(SEP_CHARS)
        .map(str::trim)
        .filter(|x| !x.is_empty())
        .map(|x| x.to_string())
        .collect()
}

/// Parsea una colección de strings "raw" y los clasifica en Géneros conocidos (`Genre`)
/// y Estilos con dedupe case-insensitive y orden de primera aparición.
fn parse_genre_style_from_raw<I, S>(raw: I) -> (Vec<Genre>, Vec<Style>)
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut seen_ci = HashSet::<String>::with_capacity(8);
    let mut parts = Vec::<String>::with_capacity(8);

    for raw_s in raw {
        for p in split_tag_field(raw_s.as_ref()) {
            if seen_ci.insert(p.to_lowercase()) {
                parts.push(p);
            }
        }
    }

    let mut genres = Vec::<Genre>::new();
    let mut styles = Vec::<Style>::new();

    for s in parts {
        if let Ok(g) = Genre::from_str(&s) {
            genres.push(g);
        } else {
            styles.push(Style::from_str(&s).unwrap());
        }
    }

    (genres, styles)
}

/// API principal: obtiene género(s) y estilo(s) a partir de un `Tag` de `lofty`.
pub fn get_genre_and_style(tag: &Tag) -> (Vec<Genre>, Vec<Style>) {
    let raw_iter = tag
        .genre()
        .into_iter()
        .map(Cow::into_owned)
        .chain(tag.get_strings(&ItemKey::Genre).map(String::from))
        .chain(get_unknown_strings(tag, "STYLE"))
        .chain(get_unknown_strings(tag, "STYLES"));

    parse_genre_style_from_raw(raw_iter)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::Style;

    fn has_style(s: &[Style], known: Style, text: &str) -> bool {
        s.iter()
            .any(|st| st == &known || matches!(st, Style::Custom(t) if t == text))
    }

    #[test]
    fn splits_common_separators() {
        let raw = vec!["Pop; Rock, Hip-Hop / R&B"];
        let (genres, styles) = parse_genre_style_from_raw(raw);
        // Asumimos que al menos "Pop" y "Rock" existen en Genre::from_str,
        // y si no, caerán como Style::Custom; el conteo valida la separación.
        assert_eq!(genres.len() + styles.len(), 4);
    }

    #[test]
    fn splits_dash_with_spaces_ascii() {
        let raw = vec!["Pop - Dance-Pop"];
        let (g, s) = parse_genre_style_from_raw(raw);
        // “Dance-Pop” puede no estar en Genre: debería quedar como Custom si no existe.
        assert!(s.iter().any(|st| matches!(st, Style::Custom(t) if t == "Dance-Pop")));
        // Si Genre::Pop existe, debe detectarse.
        assert!(g.iter().any(|g| matches!(g, Genre::Pop)));
    }

    #[test]
    fn splits_unicode_en_dash_with_spaces() {
        // EN DASH (–) con espacios
        let raw = vec!["Pop – Dance-Pop"];
        let (_g, s) = parse_genre_style_from_raw(raw);
        assert!(s.iter().any(|st| matches!(st, Style::Custom(t) if t == "Dance-Pop")));
    }

    #[test]
    fn splits_unicode_em_dash_with_spaces() {
        // EM DASH (—) con espacios
        let raw = vec!["Rock — Alternative"];
        let (_g, s) = parse_genre_style_from_raw(raw);
        assert!(s.iter().any(|st| matches!(st, Style::Custom(t) if t == "Alternative")));
    }

    #[test]
    fn does_not_split_plain_hyphen_without_spaces() {
        // No debe partir "Synth-Pop" (sin espacios)
        let raw = vec!["Pop,Synth-Pop"];
        let (_g, s) = parse_genre_style_from_raw(raw);
        assert!(
            s.iter()
                .any(|st| matches!(st, Style::SynthPop) || matches!(st, Style::Custom(t) if t == "Synth-Pop"))
        );
    }

    #[test]
    fn trims_and_dedup_case_insensitive() {
        let raw = vec!["  Pop  ", "pop", "POP"];
        let (g, s) = parse_genre_style_from_raw(raw);
        // Solo 1 entrada después de dedupe (sea Genre::Pop o Custom("Pop"))
        assert_eq!(g.len() + s.len(), 1);
    }

    #[test]
    fn splits_unicode_separators_and_nulls() {
        let raw = vec!["City Pop・Anime\0J-Pop", "Anime", "Vocaloid"];
        let (_g, s) = parse_genre_style_from_raw(raw);

        let has_city_pop = s.iter().any(|st| matches!(st, Style::Custom(t) if t == "City Pop"));
        let has_anime = s.iter().any(|st| matches!(st, Style::Custom(t) if t == "Anime"));
        let has_jpop = s.iter().any(|st| matches!(st, Style::Custom(t) if t == "J-Pop"));
        let has_vocaloid = has_style(&s, Style::Vocaloid, "Vocaloid");

        assert!(has_city_pop && has_anime && has_jpop && has_vocaloid);
    }

    #[test]
    fn preserves_first_seen_casing() {
        let raw = vec!["city pop", "City Pop"];
        let (_g, s) = parse_genre_style_from_raw(raw);
        // La forma guardada debe ser la primera (“city pop”)
        assert!(matches!(&s[0], Style::Custom(t) if t == "city pop"));
    }

    #[test]
    fn keeps_order_of_first_appearance() {
        // Usamos etiquetas no estándar para evitar que caigan en `Genre`
        let raw = vec!["Foo, Bar, Foo, Baz, Bar"];
        let (_g, s) = parse_genre_style_from_raw(raw);
        let list: Vec<&str> = s
            .iter()
            .filter_map(|st| {
                if let Style::Custom(t) = st {
                    Some(t.as_str())
                } else {
                    None
                }
            })
            .collect();
        // Orden esperado: Foo, Bar, Baz (sin duplicados)
        assert_eq!(list, vec!["Foo", "Bar", "Baz"]);
    }

    #[test]
    fn handles_nbsp_fullwidth_and_slashes() {
        // NBSP (U+00A0) y slash fullwidth (U+FF0F) -> NFKC + \s+ + split
        let raw = vec!["Pop\u{00A0}-\u{00A0}Dance-Pop ／ City Pop"];
        let (_g, s) = parse_genre_style_from_raw(raw);
        assert!(s.iter().any(|st| matches!(st, Style::Custom(t) if t == "Dance-Pop")));
        assert!(s.iter().any(|st| matches!(st, Style::Custom(t) if t == "City Pop")));
    }

    #[test]
    fn handles_fullwidth_commas_and_semicolons() {
        // Coma y punto y coma fullwidth: "，" y "；"
        let raw = vec!["Rock，Pop；Jazz"];
        let (g, s) = parse_genre_style_from_raw(raw);

        // Colecta nombres vengan de donde vengan
        let mut set = std::collections::HashSet::new();

        // Para `Genre`, usamos Display/ToString (si lo implementas; si no, adapta abajo)
        for ge in g {
            set.insert(ge.to_string());
        }
        for st in s {
            if let Style::Custom(t) = st {
                set.insert(t);
            } else {
                set.insert(st.to_string()); // por si algún día estilos conocidos
            }
        }

        assert!(set.contains("Rock") && set.contains("Pop") && set.contains("Jazz"));
    }
}
