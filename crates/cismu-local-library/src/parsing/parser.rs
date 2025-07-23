use once_cell::sync::Lazy;
use regex::Regex;

static FEAT_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\s+feat\.\s+").unwrap());

pub fn parse_performers(raw_artist_string: &str) -> (Vec<String>, Vec<String>) {
    if raw_artist_string.is_empty() {
        return (vec![], vec![]);
    }

    let parts: Vec<&str> = FEAT_REGEX.splitn(raw_artist_string, 2).collect();

    let main_artists_str = parts.get(0).unwrap_or(&"").trim();
    let featured_artists_str = parts.get(1).unwrap_or(&"").trim();

    let main_artists = if main_artists_str.is_empty() {
        vec![]
    } else {
        vec![main_artists_str.to_string()]
    };

    let featured_artists = if featured_artists_str.is_empty() {
        vec![]
    } else {
        vec![featured_artists_str.to_string()]
    };

    (main_artists, featured_artists)
}

pub fn get_raw_credits(raw_list_str: &str) -> Vec<String> {
    if raw_list_str.is_empty() {
        return vec![];
    }
    vec![raw_list_str.trim().to_string()]
}
