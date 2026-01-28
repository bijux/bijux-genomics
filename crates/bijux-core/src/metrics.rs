use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetricSet<T> {
    pub metrics_schema: String,
    pub version: i32,
    pub metrics: T,
}

impl<T> MetricSet<T> {
    #[must_use]
    pub fn new(metrics_schema: String, version: i32, metrics: T) -> Self {
        Self {
            metrics_schema,
            version,
            metrics,
        }
    }
}

pub type MetricEnvelope<T> = MetricSet<T>;
