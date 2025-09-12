use std::fmt;
use thiserror::Error;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Error del dominio para construir valores de rating válidos.
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
pub enum RatingError {
    #[error("rating fuera de rango [0.0, 5.0]")]
    OutOfRange,
}

/// Un rating que puede estar ausente (`Unrated`) o presente (`Rated`).
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Rating {
    #[default]
    Unrated,
    Rated(RatingValue),
}

impl Rating {
    /// Creador ergonómico: *clamp* a [0, 5].
    pub fn new(value: f32) -> Self {
        let v = value.clamp(0.0, 5.0);
        // `unwrap()` es seguro porque el valor ya está en rango.
        Self::Rated(RatingValue::try_new(v).unwrap())
    }

    /// Creador seguro: error si está fuera de [0, 5].
    pub fn try_new(value: f32) -> Result<Self, RatingError> {
        RatingValue::try_new(value).map(Self::Rated)
    }

    /// ¿Tiene rating?
    pub fn is_rated(&self) -> bool {
        matches!(self, Rating::Rated(_))
    }

    /// Acceso opcional al float [0, 5].
    pub fn as_f32(&self) -> Option<f32> {
        match self {
            Rating::Rated(r) => Some(r.as_f32()),
            _ => None,
        }
    }

    /// Acceso interno al valor escalado (por precisión o agregación interna).
    pub(crate) fn inner(&self) -> Option<RatingValue> {
        match self {
            Rating::Rated(r) => Some(*r),
            _ => None,
        }
    }
}

impl From<Option<RatingValue>> for Rating {
    fn from(o: Option<RatingValue>) -> Self {
        o.map_or(Self::Unrated, Self::Rated)
    }
}

impl fmt::Display for Rating {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Rating::Unrated => write!(f, "☆☆☆☆☆"),
            Rating::Rated(r) => write!(f, "{r}"),
        }
    }
}

/// Valor de rating **válido** y siempre dentro de rango.
/// Internamente se guarda como entero escalado para evitar problemas de float.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RatingValue(u32);

impl RatingValue {
    pub const SCALE_FACTOR: u32 = 10_000;
    pub const MAX_VALUE: u32 = 5 * Self::SCALE_FACTOR;

    /// Construye desde un `f32` en [0, 5].
    pub fn try_new(value: f32) -> Result<Self, RatingError> {
        if !(0.0..=5.0).contains(&value) {
            return Err(RatingError::OutOfRange);
        }
        let scaled = (value * Self::SCALE_FACTOR as f32).round() as u32;
        if scaled > Self::MAX_VALUE {
            return Err(RatingError::OutOfRange);
        }
        Ok(Self(scaled))
    }

    /// Construye desde el valor ya escalado (u32). Devuelve `None` si excede `MAX_VALUE`.
    pub fn from_scaled_u32(s: u32) -> Option<Self> {
        (s <= Self::MAX_VALUE).then_some(Self(s))
    }

    /// Valor entero escalado (ej.: 4.5 → 45000).
    pub fn scaled(self) -> u32 {
        self.0
    }

    /// Representación `f32` (0.0..=5.0).
    pub fn as_f32(self) -> f32 {
        self.0 as f32 / Self::SCALE_FACTOR as f32
    }
}

impl fmt::Display for RatingValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let full = self.as_f32().round() as usize;
        for _ in 0..full {
            write!(f, "★")?;
        }
        for _ in 0..(5 - full) {
            write!(f, "☆")?;
        }
        Ok(())
    }
}

/// Promedio de ratings con conteo, pensado para agregación incremental.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AvgRating {
    None,
    Some { mean: RatingValue, count: u32 },
}

impl AvgRating {
    pub fn none() -> Self {
        Self::None
    }

    /// Agrega un iterador de `Rating` (ignora `Unrated`).
    pub fn from_iter<I: IntoIterator<Item = Rating>>(it: I) -> Self {
        let mut sum: u128 = 0;
        let mut n: u32 = 0;
        for r in it.into_iter().filter_map(|r| r.inner()) {
            sum += r.scaled() as u128;
            n += 1;
        }
        if n == 0 {
            Self::None
        } else {
            // Redondeo "half up" en enteros: (sum + n/2) / n
            let mean_scaled = ((sum + (n as u128) / 2) / (n as u128)) as u32;
            Self::Some {
                mean: RatingValue::from_scaled_u32(mean_scaled).unwrap(),
                count: n,
            }
        }
    }

    /// Agrega un `Rating` a un promedio existente.
    pub fn add(self, r: Rating) -> Self {
        let Some(ir) = r.inner() else {
            return self;
        };
        match self {
            Self::None => Self::Some { mean: ir, count: 1 },
            Self::Some { mean, count } => {
                let sum = mean.scaled() as u128 * count as u128 + ir.scaled() as u128;
                let n = count + 1;
                let mean_scaled = ((sum + (n as u128) / 2) / (n as u128)) as u32;
                Self::Some {
                    mean: RatingValue::from_scaled_u32(mean_scaled).unwrap(),
                    count: n,
                }
            }
        }
    }

    /// Combina dos promedios pre-agregados.
    pub fn combine(self, other: Self) -> Self {
        match (self, other) {
            (Self::None, x) | (x, Self::None) => x,
            (Self::Some { mean: m1, count: n1 }, Self::Some { mean: m2, count: n2 }) => {
                let sum = m1.scaled() as u128 * n1 as u128 + m2.scaled() as u128 * n2 as u128;
                let n = n1 + n2;
                let mean_scaled = ((sum + (n as u128) / 2) / (n as u128)) as u32;
                Self::Some {
                    mean: RatingValue::from_scaled_u32(mean_scaled).unwrap(),
                    count: n,
                }
            }
        }
    }

    pub fn count(&self) -> u32 {
        match *self {
            Self::Some { count, .. } => count,
            _ => 0,
        }
    }

    /// Promedio como `Rating` público (mantiene la semántica de "unrated" si `n=0`).
    pub fn mean(&self) -> Rating {
        match *self {
            Self::None => Rating::Unrated,
            Self::Some { mean, .. } => Rating::Rated(mean),
        }
    }
}

impl fmt::Display for AvgRating {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            AvgRating::None => write!(f, "☆☆☆☆☆ (n=0)"),
            AvgRating::Some { mean, count } => write!(f, "{mean} (n={count})"),
        }
    }
}
