//! Owner: bijux-dna-analyze
//! Dashboard output model (versioned).
//! Owns dashboard-facing types only.
//! Must not perform IO or depend on report/decision layers.
//! Invariants: rows are deterministically ordered and include provenance.

use serde::{Deserialize, Serialize};

use crate::model::JsonBlob;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DashboardFactRow {
    pub schema_version: String,
    pub run_id: String,
    pub stage_id: String,
    pub tool_id: String,
    pub tool_version: String,
    pub image_digest: Option<String>,
    pub params_hash: String,
    pub input_hash: String,
    pub runtime_s: f64,
    pub memory_mb: f64,
    pub exit_code: i32,
    pub bank_hashes: JsonBlob,
    pub metrics: JsonBlob,
    pub reports: JsonBlob,
    pub artifacts: JsonBlob,
    pub trace_id: String,
    pub span_id: String,
}

impl DashboardFactRow {
    #[must_use]
    pub fn key(&self) -> (&str, &str, &str, &str, &str) {
        (
            self.run_id.as_str(),
            self.stage_id.as_str(),
            self.tool_id.as_str(),
            self.params_hash.as_str(),
            self.input_hash.as_str(),
        )
    }
}
