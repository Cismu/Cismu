use std::{fmt, str::FromStr};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Clone, Copy)]
pub enum StyleError {
    #[error("Style cannot be empty!")]
    InvalidInput,
}

/// Style is basically the same as a sub-genre.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(rename_all = "snake_case"))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Style {
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
    /// Miku Miku Beam!!
    Vocaloid,
    Custom(std::string::String),
}

impl FromStr for Style {
    type Err = StyleError;

    /// Tries to parse a style from a string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(StyleError::InvalidInput);
        }

        let normalized = trimmed.to_lowercase().replace(['-', ' '], "");

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
            "vocaloid" => Style::Vocaloid,
            _ => Style::Custom(trimmed.to_string()), // Se usa `s`, la variable sin normalizar.
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
            Style::Vocaloid => write!(f, "Vocaloid"),
            Style::Custom(s) => write!(f, "{}", s),
        }
    }
}
