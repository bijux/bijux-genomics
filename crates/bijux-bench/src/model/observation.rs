//! Owner: bijux-bench
//! Typed benchmark observations and invariants.
//! Owns bench core data model only.
//! Must not perform IO or depend on compare/gate logic.
//! Invariants: observations are fully typed and include confounders.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::error::BenchError;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetricsEnvelope {
    pub stage_id: String,
    pub schema_version: String,
    pub values: BTreeMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkObservation {
    pub schema_version: String,
    pub run_id: String,
    pub dataset_id: String,
    pub dataset_class: String,
    pub read_layout: String,
    pub stage_id: String,
    pub tool_id: String,
    pub tool_version: String,
    pub image_digest: String,
    pub params_hash: String,
    pub input_hash: String,
    pub runtime_s: f64,
    pub memory_mb: f64,
    pub exit_code: i32,
    pub failure_kind: Option<String>,
    pub metrics: MetricsEnvelope,
    pub replicate_id: String,
    pub replicate_index: u32,
    pub runner: String,
    pub platform: String,
    pub cpu: String,
    pub threads: u32,
    pub io_mode: String,
}

impl BenchmarkObservation {
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        run_id: String,
        dataset_id: String,
        dataset_class: String,
        read_layout: String,
        stage_id: String,
        tool_id: String,
        tool_version: String,
        image_digest: String,
        params_hash: String,
        input_hash: String,
        runtime_s: f64,
        memory_mb: f64,
        exit_code: i32,
        failure_kind: Option<String>,
        metrics: MetricsEnvelope,
        replicate_id: String,
        replicate_index: u32,
        runner: String,
        platform: String,
        cpu: String,
        threads: u32,
        io_mode: String,
    ) -> Self {
        Self {
            schema_version: "bijux.bench.observation.v1".to_string(),
            run_id,
            dataset_id,
            dataset_class,
            read_layout,
            stage_id,
            tool_id,
            tool_version,
            image_digest,
            params_hash,
            input_hash,
            runtime_s,
            memory_mb,
            exit_code,
            failure_kind,
            metrics,
            replicate_id,
            replicate_index,
            runner,
            platform,
            cpu,
            threads,
            io_mode,
        }
    }

    /// # Errors
    /// Returns an error if required confounders are missing.
    pub fn validate(&self) -> Result<(), BenchError> {
        if self.schema_version.trim().is_empty() {
            return Err(BenchError::InvalidObservation {
                reason: "missing schema_version".to_string(),
            });
        }
        if self.dataset_class.trim().is_empty() {
            return Err(BenchError::MissingConfounder {
                field: "dataset_class",
            });
        }
        if self.read_layout.trim().is_empty() {
            return Err(BenchError::MissingConfounder {
                field: "read_layout",
            });
        }
        if self.platform.trim().is_empty() {
            return Err(BenchError::MissingConfounder { field: "platform" });
        }
        if self.cpu.trim().is_empty() {
            return Err(BenchError::MissingConfounder { field: "cpu" });
        }
        if self.threads == 0 {
            return Err(BenchError::MissingConfounder { field: "threads" });
        }
        if self.runner.trim().is_empty() {
            return Err(BenchError::MissingConfounder { field: "runner" });
        }
        if self.io_mode.trim().is_empty() {
            return Err(BenchError::MissingConfounder { field: "io_mode" });
        }
        if self.image_digest.trim().is_empty() {
            return Err(BenchError::MissingConfounder {
                field: "image_digest",
            });
        }
        Ok(())
    }
}
