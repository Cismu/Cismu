use lofty::tag::{ItemKey, Tag};
use temporal_rs::partial::PartialDate;

/// Intenta parsear una fecha parcial desde un &str en formato "YYYY[-MM[-DD]]".
fn parse_partial_date(s: &str) -> Option<PartialDate> {
    let parts: Vec<&str> = s.trim().split('-').collect();
    let mut pd = PartialDate::new();
    match parts.as_slice() {
        [year] => {
            if let Ok(y) = year.parse() {
                pd = pd.with_year(Some(y));
                return Some(pd);
            }
        }
        [year, month] => {
            if let (Ok(y), Ok(m)) = (year.parse(), month.parse()) {
                pd = pd.with_year(Some(y)).with_month(Some(m));
                return Some(pd);
            }
        }
        [year, month, day] => {
            if let (Ok(y), Ok(m), Ok(d)) = (year.parse(), month.parse(), day.parse()) {
                pd = pd.with_year(Some(y)).with_month(Some(m)).with_day(Some(d));
                return Some(pd);
            }
        }
        _ => {}
    }
    None
}

/// Puntuación para comparar fechas parciales: día = 4, mes = 2, año = 1.
fn score(pd: &PartialDate) -> u8 {
    (pd.calendar_fields.day.is_some() as u8) * 4
        + (pd.calendar_fields.month.is_some() as u8) * 2
        + 1
}

/// Extrae valores Unknown(key) de un Tag (case-insensitive).
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

/// Trait para extraer la mejor PartialDate de un Tag usando múltiples heurísticas.
pub trait PartialDateFromTag {
    /// Revisa, en orden:
    /// - `tag.get_strings(&key)`
    /// - `tag.get_string(&key)`
    /// - valores Unknown(key)
    /// Devuelve la fecha parcial con mayor nivel de detalle.
    fn extract_partial_date(&self, key: ItemKey) -> Option<PartialDate>;
}

impl PartialDateFromTag for Tag {
    fn extract_partial_date(&self, key: ItemKey) -> Option<PartialDate> {
        // 1) Reunir todas las cadenas candidatas en un Vec<&str>
        let mut candidates: Vec<&str> = self.get_strings(&key).map(|cow| cow.as_ref()).collect();
        if let Some(cow) = self.get_string(&key) {
            candidates.push(cow.as_ref());
        }
        if let ItemKey::Unknown(ref k) = key {
            candidates.extend(get_all_unknown(self, k));
        }

        // 2) Parsear y seleccionar la PartialDate más completa
        candidates
            .into_iter()
            .filter_map(parse_partial_date)
            .max_by(|a, b| score(a).cmp(&score(b)))
    }
}
