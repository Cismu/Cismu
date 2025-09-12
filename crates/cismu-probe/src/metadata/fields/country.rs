use celes::Country as CelesCountry;
use std::ops::Deref;
use std::str::FromStr;
use thiserror::Error;

#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum CountryError {
    #[error("The country `{0}` is invalid or could not be found.")]
    Invalid(String),
}

#[derive(Debug, Clone, Copy)]
pub struct Country(CelesCountry);

impl FromStr for Country {
    type Err = CountryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Usamos la función de búsqueda de `celes`
        CelesCountry::from_str(s)
            .map(Country)
            .map_err(|_| CountryError::Invalid(s.to_string()))
    }
}

impl Deref for Country {
    type Target = CelesCountry;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(feature = "serde")]
impl Serialize for Country {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serializamos el país como su código alpha2
        serializer.serialize_str(self.0.alpha2)
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Country {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        // Deserializamos un string; usamos nuestra lógica de FromStr para encontrar el país
        Country::from_str(&s).map_err(serde::de::Error::custom)
    }
}
