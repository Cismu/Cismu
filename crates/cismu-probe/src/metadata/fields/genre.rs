use std::{fmt, str::FromStr};
use thiserror::Error;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum GenreError {
    #[error("The type {0} is invalid; only variants of the enum are allowed.")]
    Invalid(String),
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Genre {
    Rock,
    Electronic,
    Pop,
    FolkWorldAndCountry,
    Jazz,
    FunkSoul,
    Classical,
    HipHop,
    Latin,
    StageAndScreen,
    Reggae,
    Blues,
    NonMusic,
    Childrens,
    BrassAndMilitary,
}

impl fmt::Display for Genre {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Genre::Rock => "Rock",
            Genre::Electronic => "Electronic",
            Genre::Pop => "Pop",
            Genre::FolkWorldAndCountry => "Folk, World, & Country",
            Genre::Jazz => "Jazz",
            Genre::FunkSoul => "Funk / Soul",
            Genre::Classical => "Classical",
            Genre::HipHop => "Hip Hop",
            Genre::Latin => "Latin",
            Genre::StageAndScreen => "Stage & Screen",
            Genre::Reggae => "Reggae",
            Genre::Blues => "Blues",
            Genre::NonMusic => "Non-Music",
            Genre::Childrens => "Children's",
            Genre::BrassAndMilitary => "Brass & Military",
        };
        write!(f, "{}", text)
    }
}

impl FromStr for Genre {
    type Err = GenreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalized = s.to_lowercase().replace(['-', ' ', ',', '&', '/'], "");

        match normalized.as_str() {
            "rock" => Ok(Genre::Rock),
            "electronic" => Ok(Genre::Electronic),
            "pop" => Ok(Genre::Pop),
            "folkworldandcountry" | "folkworldcountry" => Ok(Genre::FolkWorldAndCountry),
            "jazz" => Ok(Genre::Jazz),
            "funksoul" => Ok(Genre::FunkSoul),
            "classical" => Ok(Genre::Classical),
            "hiphop" => Ok(Genre::HipHop),
            "latin" => Ok(Genre::Latin),
            "stageandscreen" | "stagescreen" => Ok(Genre::StageAndScreen),
            "reggae" => Ok(Genre::Reggae),
            "blues" => Ok(Genre::Blues),
            "nonmusic" => Ok(Genre::NonMusic),
            "childrens" | "children" => Ok(Genre::Childrens),
            "brassandmilitary" | "brassmilitary" => Ok(Genre::BrassAndMilitary),
            _ => Err(GenreError::Invalid(s.to_string())), // Se usa el string original 's'
        }
    }
}
