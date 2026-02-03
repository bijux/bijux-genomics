use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use thiserror::Error;
use tracing::warn;

pub type Result<T> = std::result::Result<T, BenchError>;

pub trait StageMetricSchema {
    const STAGE: &'static str;
    /// Validate the schema invariants.
    ///
    /// # Errors
    /// Returns an error if the schema invariants are violated.
    fn validate(&self) -> Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkContext {
    pub tool: String,
    pub tool_version: String,
    pub image_digest: String,
    pub runner: String,
    pub platform: String,
    pub input_hash: String,
    pub parameters: JsonValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionMetrics {
    pub runtime_s: f64,
    pub memory_mb: f64,
    pub exit_code: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqTrimMetrics {
    pub reads_in: u64,
    pub reads_out: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    pub mean_q_before: f64,
    pub mean_q_after: f64,
}

impl StageMetricSchema for FastqTrimMetrics {
    const STAGE: &'static str = "fastq.trim";

    fn validate(&self) -> Result<()> {
        if self.reads_out > self.reads_in {
            return Err(BenchError::Validation(
                "reads_out must be <= reads_in".to_string(),
            ));
        }
        if self.bases_out > self.bases_in {
            return Err(BenchError::Validation(
                "bases_out must be <= bases_in".to_string(),
            ));
        }
        if self.mean_q_after < self.mean_q_before {
            warn!(
                mean_q_before = self.mean_q_before,
                mean_q_after = self.mean_q_after,
                "mean_q_after is lower than mean_q_before"
            );
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkRecord<T: StageMetricSchema> {
    pub context: BenchmarkContext,
    pub execution: ExecutionMetrics,
    pub metrics: T,
}

impl<T> BenchmarkRecord<T>
where
    T: StageMetricSchema,
{
    /// Validate the record by validating its metrics.
    ///
    /// # Errors
    /// Returns an error if the metric schema validation fails.
    pub fn validate(&self) -> Result<()> {
        self.metrics.validate()
    }
}

#[derive(Debug, Error)]
pub enum BenchError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("validation error: {0}")]
    Validation(String),
}

/// Append a benchmark record as a JSONL line.
///
/// # Errors
/// Returns an error if the file cannot be written.
pub fn append_jsonl<T>(path: &Path, record: &BenchmarkRecord<T>) -> Result<()>
where
    T: StageMetricSchema + Serialize,
{
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    let line = serde_json::to_string(record)?;
    writeln!(file, "{line}")?;
    Ok(())
}

pub const FASTQ_TRIM_SCHEMA_VERSION: i32 = 1;
pub const FASTQ_VALIDATE_SCHEMA_VERSION: i32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqValidateMetrics {
    pub reads: u64,
    pub bases: u64,
    pub mean_q: f64,
    pub format_valid: bool,
}

impl StageMetricSchema for FastqValidateMetrics {
    const STAGE: &'static str = "fastq.validate";

    fn validate(&self) -> Result<()> {
        Ok(())
    }
}

/// Open a `SQLite` connection for benchmark persistence.
///
/// # Errors
/// Returns an error if the connection cannot be opened.
pub fn open_sqlite(path: &Path) -> Result<Connection> {
    Ok(Connection::open(path)?)
}

/// Insert a `FastQ` trim benchmark record into the v1 table.
///
/// # Errors
/// Returns an error if the table cannot be created or the record cannot be inserted.
pub fn insert_fastq_trim_v1(
    conn: &Connection,
    record: &BenchmarkRecord<FastqTrimMetrics>,
) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS bench_fastq_trim_v1 (\
         tool TEXT NOT NULL,\
         tool_version TEXT NOT NULL,\
         image_digest TEXT NOT NULL,\
         runner TEXT NOT NULL,\
         platform TEXT NOT NULL,\
         input_hash TEXT NOT NULL,\
         parameters_json TEXT NOT NULL,\
         schema_version INTEGER NOT NULL,\
         runtime_s REAL NOT NULL,\
         memory_mb REAL NOT NULL,\
         exit_code INTEGER NOT NULL,\
         metrics_json TEXT NOT NULL\
         )",
        [],
    )?;

    let metrics_json = serde_json::to_string(&record.metrics)?;
    let parameters_json = serde_json::to_string(&record.context.parameters)?;

    conn.execute(
        "INSERT INTO bench_fastq_trim_v1 (\
         tool, tool_version, image_digest, runner, platform, input_hash,\
         parameters_json, schema_version, runtime_s, memory_mb, exit_code, metrics_json\
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        (
            &record.context.tool,
            &record.context.tool_version,
            &record.context.image_digest,
            &record.context.runner,
            &record.context.platform,
            &record.context.input_hash,
            parameters_json,
            FASTQ_TRIM_SCHEMA_VERSION,
            record.execution.runtime_s,
            record.execution.memory_mb,
            record.execution.exit_code,
            metrics_json,
        ),
    )?;
    Ok(())
}

/// Insert a `FastQ` validate benchmark record into the v1 table.
///
/// # Errors
/// Returns an error if the table cannot be created or the record cannot be inserted.
pub fn insert_fastq_validate_v1(
    conn: &Connection,
    record: &BenchmarkRecord<FastqValidateMetrics>,
) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS bench_fastq_validate_v1 (\
         tool TEXT NOT NULL,\
         tool_version TEXT NOT NULL,\
         image_digest TEXT NOT NULL,\
         runner TEXT NOT NULL,\
         platform TEXT NOT NULL,\
         input_hash TEXT NOT NULL,\
         parameters_json TEXT NOT NULL,\
         schema_version INTEGER NOT NULL,\
         runtime_s REAL NOT NULL,\
         memory_mb REAL NOT NULL,\
         exit_code INTEGER NOT NULL,\
         metrics_json TEXT NOT NULL\
         )",
        [],
    )?;

    let metrics_json = serde_json::to_string(&record.metrics)?;
    let parameters_json = serde_json::to_string(&record.context.parameters)?;

    conn.execute(
        "INSERT INTO bench_fastq_validate_v1 (\
         tool, tool_version, image_digest, runner, platform, input_hash,\
         parameters_json, schema_version, runtime_s, memory_mb, exit_code, metrics_json\
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        (
            &record.context.tool,
            &record.context.tool_version,
            &record.context.image_digest,
            &record.context.runner,
            &record.context.platform,
            &record.context.input_hash,
            parameters_json,
            FASTQ_VALIDATE_SCHEMA_VERSION,
            record.execution.runtime_s,
            record.execution.memory_mb,
            record.execution.exit_code,
            metrics_json,
        ),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_serializes() -> Result<()> {
        let record = BenchmarkRecord {
            context: BenchmarkContext {
                tool: "fastp".to_string(),
                tool_version: "0.23.4".to_string(),
                image_digest: "sha256:abc".to_string(),
                runner: "docker".to_string(),
                platform: "local".to_string(),
                input_hash: "sha256:deadbeef".to_string(),
                parameters: serde_json::json!({"adapter": "AGAT"}),
            },
            execution: ExecutionMetrics {
                runtime_s: 1.0,
                memory_mb: 10.0,
                exit_code: 0,
            },
            metrics: FastqTrimMetrics {
                reads_in: 100,
                reads_out: 90,
                bases_in: 1000,
                bases_out: 900,
                mean_q_before: 30.0,
                mean_q_after: 31.0,
            },
        };
        record.validate()?;
        let json = serde_json::to_string(&record)?;
        assert!(json.contains("fastp"));
        Ok(())
    }
}
