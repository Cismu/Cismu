use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AvgRating {
    Unrated,
    Rated(Rating),
}

impl Default for AvgRating {
    fn default() -> Self {
        AvgRating::Unrated
    }
}

impl fmt::Display for AvgRating {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AvgRating::Unrated => write!(f, "☆☆☆☆☆"),
            AvgRating::Rated(rating) => fmt::Display::fmt(rating, f),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Rating(u32);

impl Rating {
    const SCALE_FACTOR: u32 = 10000;
    const MAX_VALUE: u32 = 5 * Self::SCALE_FACTOR;

    pub fn new(value: f32) -> Option<Self> {
        if !(0.0..=5.0).contains(&value) {
            return None;
        }

        let scaled_value = (value * Self::SCALE_FACTOR as f32).round() as u32;

        if scaled_value > Self::MAX_VALUE {
            return None;
        }

        Some(Self(scaled_value))
    }

    pub fn as_f32(&self) -> f32 {
        self.0 as f32 / Self::SCALE_FACTOR as f32
    }
}

impl fmt::Display for Rating {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let full_stars = self.as_f32().round() as usize;
        let empty_stars = 5 - full_stars;

        for _ in 0..full_stars {
            write!(f, "★")?;
        }
        for _ in 0..empty_stars {
            write!(f, "☆")?;
        }

        Ok(())
    }
}
