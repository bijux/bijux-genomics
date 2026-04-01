use serde::de::Deserializer;
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};

use super::DefaultParams;

mod deserialize;
mod serialize;

impl DefaultParams {
    #[must_use]
    pub fn to_json(&self) -> serde_json::Value {
        serialize::to_json(self)
    }
}

impl Serialize for DefaultParams {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.to_json().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for DefaultParams {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        deserialize::from_json(value).map_err(serde::de::Error::custom)
    }
}
