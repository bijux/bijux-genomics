//! Owner: bijux-analyze
//! Benchmark records and JSONL appenders.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use bijux_core::measure::ExecutionMetrics;
use bijux_core::metrics::MetricSet;
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
