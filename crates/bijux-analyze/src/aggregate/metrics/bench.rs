//! Owner: bijux-analyze
//! Benchmark records and image QA schemas.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use bijux_core::metrics::MetricSet;
use bijux_core::primitives::measure::ExecutionMetrics;
use serde::{Deserialize, Serialize};

use crate::aggregate::{validate_metric_set, Result, StageMetricSchema};
use crate::model::JsonBlob;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkContext {
    pub tool: String,
    pub tool_version: String,
    pub image_digest: String,
    pub runner: String,
    pub platform: String,
    pub input_hash: String,
    pub parameters: JsonBlob,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkRecord<T: StageMetricSchema> {
    pub context: BenchmarkContext,
    pub execution: ExecutionMetrics,
    pub metrics: MetricSet<T>,
}

impl<T> BenchmarkRecord<T>
where
    T: StageMetricSchema + Serialize,
{
    /// Validate the record by validating its metrics.
    ///
    /// # Errors
    /// Returns an error if the metric schema validation fails.
    pub fn validate(&self) -> Result<()> {
        self.execution.validate()?;
        validate_metric_set(&self.metrics)
    }
}

/// Append a benchmark record to a JSONL file.
///
/// # Errors
/// Returns an error if the file cannot be opened or the record cannot be serialized.
pub fn append_jsonl<T>(path: &Path, record: &BenchmarkRecord<T>) -> Result<()>
where
    T: StageMetricSchema + Serialize,
{
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    let line = serde_json::to_string(record)?;
    writeln!(file, "{line}")?;
    Ok(())
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ImageQaOutcome {
    Pass,
    Fail(String),
}

impl ImageQaOutcome {
    #[must_use]
    pub fn status(&self) -> &'static str {
        match self {
            ImageQaOutcome::Pass => "pass",
            ImageQaOutcome::Fail(_) => "fail",
        }
    }

    #[must_use]
    pub fn failure_reason(&self) -> Option<&str> {
        match self {
            ImageQaOutcome::Pass => None,
            ImageQaOutcome::Fail(reason) => Some(reason.as_str()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImageQaRecord {
    pub tool: String,
    pub stage: String,
    pub tool_version: String,
    pub image_digest: String,
    pub runner: String,
    pub platform: String,
    pub input_hash: String,
    pub outcome: ImageQaOutcome,
}

/// Append a benchmark record as a JSONL line.
///
/// # Errors
/// Returns an error if the file cannot be written.
pub fn append_image_qa_jsonl(path: &Path, record: &ImageQaRecord) -> Result<()> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    let line = serde_json::to_string(record)?;
    writeln!(file, "{line}")?;
    Ok(())
}

pub const IMAGE_QA_SCHEMA_VERSION: i32 = 1;
pub const IMAGE_QA_INPUTS_SCHEMA_VERSION: i32 = 1;
