//! Owner: bijux-dna-bench-model
//! Typed metrics envelope for benchmark observations.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetricsEnvelope {
    pub stage_id: String,
    pub schema_version: String,
    pub values: BTreeMap<String, f64>,
}
