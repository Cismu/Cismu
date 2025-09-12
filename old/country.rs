use std::{collections::HashSet, convert::TryFrom, str::FromStr};

use anyhow::{Error, Result, anyhow};
use celes::Country as CountryCELES;
use lofty::tag::{ItemKey, Tag};

const KEY_COUNTRY_UNKNOWN: &str = "COUNTRY";

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Country {
    pub alpha2: String,
    pub long_name: String,
}

impl Country {
    pub fn new(alpha2: &str, long_name: &str) -> Self {
        Self {
            alpha2: alpha2.to_string(),
            long_name: long_name.to_string(),
        }
    }
}

impl FromStr for Country {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let country = CountryCELES::from_str(s).map_err(|_| anyhow!("Invalid country: {}", s))?;
        Ok(Country::new(&country.alpha2, &country.long_name))
    }
}

fn get_all_unknown_strings<'a>(tag: &'a Tag, key: &'a str) -> impl Iterator<Item = &'a str> + 'a {
    tag.items().filter_map(move |item| {
        if let ItemKey::Unknown(k) = item.key() {
            if k.eq_ignore_ascii_case(key) {
                return item.value().text();
            }
        }
        None
    })
}

impl TryFrom<&Tag> for Country {
    type Error = Error;

    fn try_from(tag: &Tag) -> Result<Self> {
        let mut candidates: Vec<&str> = Vec::new();
        candidates.extend(get_all_unknown_strings(tag, KEY_COUNTRY_UNKNOWN));

        let mut seen = HashSet::new();
        candidates.retain(|&c| seen.insert(c.to_lowercase()));

        for cand in candidates {
            if let Ok(country) = cand.parse::<Country>() {
                return Ok(country);
            }
        }

        Err(anyhow!("No valid country found in tag"))
    }
}
