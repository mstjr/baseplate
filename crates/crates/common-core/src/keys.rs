use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Key {
    Id(Uuid),
    ApiName(String),
}

#[derive(Clone, Debug, Copy)]
pub enum KeyType {
    Id,
    ApiName,
}

impl FromStr for KeyType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "id" => Ok(KeyType::Id),
            "api_name" => Ok(KeyType::ApiName),
            _ => Err(format!("Invalid key type: {}", s)),
        }
    }
}

impl ToString for KeyType {
    fn to_string(&self) -> String {
        match self {
            KeyType::Id => "id".to_string(),
            KeyType::ApiName => "api_name".to_string(),
        }
    }
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self) // or your custom formatting
    }
}

impl<'de> Deserialize<'de> for Key {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Key::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl FromStr for Key {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(id) = Uuid::parse_str(s) {
            Ok(Key::Id(id))
        } else {
            Ok(Key::ApiName(s.to_string()))
        }
    }
}

impl Serialize for Key {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Key::Id(id) => id.serialize(serializer),
            Key::ApiName(name) => name.serialize(serializer),
        }
    }
}

impl Key {
    pub fn from_parts(kt: KeyType, id: &Uuid, api_name: &str) -> Self {
        match kt {
            KeyType::Id => Key::Id(*id),
            KeyType::ApiName => Key::ApiName(api_name.to_string()),
        }
    }
}
