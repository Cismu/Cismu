use std::{fmt, str::FromStr};

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Define el género musical (se usa el modelo de Discogs).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, specta::Type)]
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
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
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
            _ => Err(anyhow::anyhow!("Invalid genre: {}", s)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, specta::Type)]
pub enum Style {
    // --- Ordenado por popularidad de Discogs ---
    PopRock,
    House,
    Vocal,
    Experimental,
    Punk,
    AlternativeRock,
    SynthPop,
    Techno,
    IndieRock,
    Ambient,
    Soul,
    Disco,
    Hardcore,
    Folk,
    Ballad,
    Country,
    HardRock,
    Electro,
    RockAndRoll,
    Chanson,
    Romantic,
    Trance,
    HeavyMetal,
    PsychedelicRock,
    FolkRock,
    Jpop,
    Vocaloid,
    Custom(std::string::String),
}

impl FromStr for Style {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalized = s.to_lowercase().replace(['-', ' '], "");

        let style = match normalized.as_str() {
            "poprock" => Style::PopRock,
            "house" => Style::House,
            "vocal" => Style::Vocal,
            "experimental" => Style::Experimental,
            "punk" => Style::Punk,
            "alternativerock" => Style::AlternativeRock,
            "synthpop" => Style::SynthPop,
            "techno" => Style::Techno,
            "indierock" => Style::IndieRock,
            "ambient" => Style::Ambient,
            "soul" => Style::Soul,
            "disco" => Style::Disco,
            "hardcore" => Style::Hardcore,
            "folk" => Style::Folk,
            "ballad" => Style::Ballad,
            "country" => Style::Country,
            "hardrock" => Style::HardRock,
            "electro" => Style::Electro,
            "rock&roll" | "rockandroll" => Style::RockAndRoll,
            "chanson" => Style::Chanson,
            "romantic" => Style::Romantic,
            "trance" => Style::Trance,
            "heavymetal" => Style::HeavyMetal,
            "psychedelicrock" => Style::PsychedelicRock,
            "folkrock" => Style::FolkRock,
            // --- Adiciones personales ---
            "jpop" => Style::Jpop,
            "vocaloid" => Style::Vocaloid,
            // El caso por defecto usa el string original 's'
            _ => Style::Custom(s.to_string()),
        };

        Ok(style)
    }
}

impl fmt::Display for Style {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Style::PopRock => write!(f, "Pop Rock"),
            Style::House => write!(f, "House"),
            Style::Vocal => write!(f, "Vocal"),
            Style::Experimental => write!(f, "Experimental"),
            Style::Punk => write!(f, "Punk"),
            Style::AlternativeRock => write!(f, "Alternative Rock"),
            Style::SynthPop => write!(f, "Synth-pop"),
            Style::Techno => write!(f, "Techno"),
            Style::IndieRock => write!(f, "Indie Rock"),
            Style::Ambient => write!(f, "Ambient"),
            Style::Soul => write!(f, "Soul"),
            Style::Disco => write!(f, "Disco"),
            Style::Hardcore => write!(f, "Hardcore"),
            Style::Folk => write!(f, "Folk"),
            Style::Ballad => write!(f, "Ballad"),
            Style::Country => write!(f, "Country"),
            Style::HardRock => write!(f, "Hard Rock"),
            Style::Electro => write!(f, "Electro"),
            Style::RockAndRoll => write!(f, "Rock & Roll"),
            Style::Chanson => write!(f, "Chanson"),
            Style::Romantic => write!(f, "Romantic"),
            Style::Trance => write!(f, "Trance"),
            Style::HeavyMetal => write!(f, "Heavy Metal"),
            Style::PsychedelicRock => write!(f, "Psychedelic Rock"),
            Style::FolkRock => write!(f, "Folk Rock"),
            // --- Adiciones personales ---
            Style::Jpop => write!(f, "J-pop"),
            Style::Vocaloid => write!(f, "Vocaloid"),
            // --- Variante por defecto ---
            Style::Custom(s) => write!(f, "{}", s),
        }
    }
}
