//! Owner: bijux-analyze
//! Typed JSON blob wrapper.

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonBlob(serde_json::Value);

impl JsonBlob {
    #[must_use]
    pub fn new(value: serde_json::Value) -> Self {
        Self(value)
    }

    #[must_use]
    pub fn as_value(&self) -> &serde_json::Value {
        &self.0
    }

    #[must_use]
    pub fn from_pairs(pairs: &[(&str, &str)]) -> Self {
        let mut map = serde_json::Map::new();
        for (key, value) in pairs {
            map.insert(
                (*key).to_string(),
                serde_json::Value::String((*value).to_string()),
            );
        }
        Self(serde_json::Value::Object(map))
    }

    /// # Errors
    /// Returns an error if the value cannot be serialized to JSON.
    pub fn from_serializable<T: Serialize>(value: &T) -> Result<Self> {
        let json = serde_json::to_value(value)?;
        Ok(Self(json))
    }

    /// # Errors
    /// Returns an error if the raw string cannot be parsed as JSON.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(raw: &str) -> Result<Self> {
        let json = serde_json::from_str(raw)?;
        Ok(Self(json))
    }

    #[must_use]
    pub fn numeric_deltas(metrics: &JsonBlob, baseline: &JsonBlob) -> Self {
        let mut out = serde_json::Map::new();
        let Some(metrics_map) = metrics.as_value().as_object() else {
            return Self(serde_json::Value::Object(out));
        };
        let Some(baseline_map) = baseline.as_value().as_object() else {
            return Self(serde_json::Value::Object(out));
        };
        for (key, value) in metrics_map {
            if let (Some(a), Some(b)) = (
                value.as_f64(),
                baseline_map.get(key).and_then(serde_json::Value::as_f64),
            ) {
                out.insert(key.clone(), serde_json::Value::from(a - b));
            }
        }
        Self(serde_json::Value::Object(out))
    }
}

impl From<serde_json::Value> for JsonBlob {
    fn from(value: serde_json::Value) -> Self {
        Self(value)
    }
}

impl Default for JsonBlob {
    fn default() -> Self {
        Self(serde_json::Value::Object(serde_json::Map::new()))
    }
}
