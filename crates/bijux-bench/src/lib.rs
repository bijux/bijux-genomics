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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageMetricKind {
    FastqTrim,
    FastqValidate,
    FastqFilter,
    FastqMerge,
}

#[derive(Debug, Clone, Copy)]
pub struct MetricDefinition {
    pub name: &'static str,
    pub meaning: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct StageMetricSpec {
    pub stage: &'static str,
    pub metrics: &'static [MetricDefinition],
    pub invariants: &'static [&'static str],
}

pub const FASTQ_TRIM_METRICS: [MetricDefinition; 6] = [
    MetricDefinition {
        name: "reads_in",
        meaning: "Number of input reads",
    },
    MetricDefinition {
        name: "reads_out",
        meaning: "Number of output reads",
    },
    MetricDefinition {
        name: "bases_in",
        meaning: "Number of input bases",
    },
    MetricDefinition {
        name: "bases_out",
        meaning: "Number of output bases",
    },
    MetricDefinition {
        name: "mean_q_before",
        meaning: "Mean Phred quality score before trimming",
    },
    MetricDefinition {
        name: "mean_q_after",
        meaning: "Mean Phred quality score after trimming",
    },
];

pub const FASTQ_VALIDATE_METRICS: [MetricDefinition; 4] = [
    MetricDefinition {
        name: "reads_total",
        meaning: "Total number of reads observed",
    },
    MetricDefinition {
        name: "reads_valid",
        meaning: "Number of reads that passed validation",
    },
    MetricDefinition {
        name: "reads_invalid",
        meaning: "Number of reads that failed validation",
    },
    MetricDefinition {
        name: "mean_q",
        meaning: "Mean Phred quality score across all bases",
    },
];

pub const FASTQ_FILTER_METRICS: [MetricDefinition; 5] = [
    MetricDefinition {
        name: "reads_in",
        meaning: "Number of input reads",
    },
    MetricDefinition {
        name: "reads_out",
        meaning: "Number of output reads",
    },
    MetricDefinition {
        name: "reads_dropped",
        meaning: "Number of reads removed by filtering",
    },
    MetricDefinition {
        name: "mean_q_before",
        meaning: "Mean Phred quality score before filtering",
    },
    MetricDefinition {
        name: "mean_q_after",
        meaning: "Mean Phred quality score after filtering",
    },
];

pub const FASTQ_MERGE_METRICS: [MetricDefinition; 5] = [
    MetricDefinition {
        name: "reads_r1",
        meaning: "Number of reads in read 1 input",
    },
    MetricDefinition {
        name: "reads_r2",
        meaning: "Number of reads in read 2 input",
    },
    MetricDefinition {
        name: "reads_merged",
        meaning: "Number of merged reads",
    },
    MetricDefinition {
        name: "reads_unmerged",
        meaning: "Number of unmerged reads (per end)",
    },
    MetricDefinition {
        name: "merge_rate",
        meaning: "Merged reads divided by min(reads_r1, reads_r2)",
    },
];

pub const FASTQ_TRIM_INVARIANTS: [&str; 4] = [
    "reads_out <= reads_in",
    "bases_out <= bases_in",
    "mean_q_after >= mean_q_before (warn)",
    "counts are non-negative",
];

pub const FASTQ_VALIDATE_INVARIANTS: [&str; 3] = [
    "reads_valid + reads_invalid == reads_total",
    "mean_q in [0, 45]",
    "counts are non-negative",
];

pub const FASTQ_FILTER_INVARIANTS: [&str; 3] = [
    "reads_out + reads_dropped == reads_in",
    "mean_q_after >= mean_q_before (warn)",
    "counts are non-negative",
];

pub const FASTQ_MERGE_INVARIANTS: [&str; 3] = [
    "reads_merged + reads_unmerged <= min(reads_r1, reads_r2)",
    "merge_rate in [0, 1]",
    "counts are non-negative",
];

#[must_use]
pub fn metric_kind_for_stage(stage_id: &str) -> Option<StageMetricKind> {
    match stage_id {
        "fastq.trim" => Some(StageMetricKind::FastqTrim),
        "fastq.validate" => Some(StageMetricKind::FastqValidate),
        "fastq.filter" => Some(StageMetricKind::FastqFilter),
        "fastq.merge" => Some(StageMetricKind::FastqMerge),
        _ => None,
    }
}

#[must_use]
pub fn stage_metric_spec(kind: StageMetricKind) -> StageMetricSpec {
    match kind {
        StageMetricKind::FastqTrim => StageMetricSpec {
            stage: "fastq.trim",
            metrics: &FASTQ_TRIM_METRICS,
            invariants: &FASTQ_TRIM_INVARIANTS,
        },
        StageMetricKind::FastqValidate => StageMetricSpec {
            stage: "fastq.validate",
            metrics: &FASTQ_VALIDATE_METRICS,
            invariants: &FASTQ_VALIDATE_INVARIANTS,
        },
        StageMetricKind::FastqFilter => StageMetricSpec {
            stage: "fastq.filter",
            metrics: &FASTQ_FILTER_METRICS,
            invariants: &FASTQ_FILTER_INVARIANTS,
        },
        StageMetricKind::FastqMerge => StageMetricSpec {
            stage: "fastq.merge",
            metrics: &FASTQ_MERGE_METRICS,
            invariants: &FASTQ_MERGE_INVARIANTS,
        },
    }
}

pub struct StageMetricRegistry;

impl StageMetricRegistry {
    #[must_use]
    pub fn kind_for_stage(stage_id: &str) -> Option<StageMetricKind> {
        metric_kind_for_stage(stage_id)
    }

    #[must_use]
    pub fn spec_for_stage(stage_id: &str) -> Option<StageMetricSpec> {
        Self::kind_for_stage(stage_id).map(stage_metric_spec)
    }
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
pub fn append_jsonl<T>(path: &Path, record: &BenchmarkRecord<T>) -> Result<()>
where
    T: StageMetricSchema + Serialize,
{
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    let line = serde_json::to_string(record)?;
    writeln!(file, "{line}")?;
    Ok(())
}

/// Append an image QA record as a JSONL line.
///
/// # Errors
/// Returns an error if the file cannot be written.
pub fn append_image_qa_jsonl(path: &Path, record: &ImageQaRecord) -> Result<()> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    let line = serde_json::to_string(record)?;
    writeln!(file, "{line}")?;
    Ok(())
}

pub const FASTQ_TRIM_SCHEMA_VERSION: i32 = 1;
pub const FASTQ_VALIDATE_SCHEMA_VERSION: i32 = 1;
pub const FASTQ_FILTER_SCHEMA_VERSION: i32 = 1;
pub const FASTQ_MERGE_SCHEMA_VERSION: i32 = 1;
pub const IMAGE_QA_SCHEMA_VERSION: i32 = 1;
pub const IMAGE_QA_INPUTS_SCHEMA_VERSION: i32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqValidateMetrics {
    pub reads_total: u64,
    pub reads_valid: u64,
    pub reads_invalid: u64,
    pub mean_q: f64,
}

impl StageMetricSchema for FastqValidateMetrics {
    const STAGE: &'static str = "fastq.validate";

    fn validate(&self) -> Result<()> {
        if self.reads_valid + self.reads_invalid != self.reads_total {
            return Err(BenchError::Validation(
                "reads_valid + reads_invalid must equal reads_total".to_string(),
            ));
        }
        if !self.mean_q.is_finite() || !(0.0..=45.0).contains(&self.mean_q) {
            return Err(BenchError::Validation(
                "mean_q must be within [0, 45]".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqFilterMetrics {
    pub reads_in: u64,
    pub reads_out: u64,
    pub reads_dropped: u64,
    pub mean_q_before: f64,
    pub mean_q_after: f64,
}

impl StageMetricSchema for FastqFilterMetrics {
    const STAGE: &'static str = "fastq.filter";

    fn validate(&self) -> Result<()> {
        if self.reads_out + self.reads_dropped != self.reads_in {
            return Err(BenchError::Validation(
                "reads_out + reads_dropped must equal reads_in".to_string(),
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
pub struct FastqMergeMetrics {
    pub reads_r1: u64,
    pub reads_r2: u64,
    pub reads_merged: u64,
    pub reads_unmerged: u64,
    pub merge_rate: f64,
}

impl StageMetricSchema for FastqMergeMetrics {
    const STAGE: &'static str = "fastq.merge";

    fn validate(&self) -> Result<()> {
        let min_reads = self.reads_r1.min(self.reads_r2);
        if self.reads_merged + self.reads_unmerged > min_reads {
            return Err(BenchError::Validation(
                "reads_merged + reads_unmerged must be <= min(reads_r1, reads_r2)".to_string(),
            ));
        }
        if !self.merge_rate.is_finite() || !(0.0..=1.0).contains(&self.merge_rate) {
            return Err(BenchError::Validation(
                "merge_rate must be within [0, 1]".to_string(),
            ));
        }
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

/// Insert a `FastQ` filter benchmark record into the v1 table.
///
/// # Errors
/// Returns an error if the table cannot be created or the record cannot be inserted.
pub fn insert_fastq_filter_v1(
    conn: &Connection,
    record: &BenchmarkRecord<FastqFilterMetrics>,
) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS bench_fastq_filter_v1 (\
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
        "INSERT INTO bench_fastq_filter_v1 (\
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
            FASTQ_FILTER_SCHEMA_VERSION,
            record.execution.runtime_s,
            record.execution.memory_mb,
            record.execution.exit_code,
            metrics_json,
        ),
    )?;
    Ok(())
}

/// Insert a `FastQ` merge benchmark record into the v1 table.
///
/// # Errors
/// Returns an error if the table cannot be created or the record cannot be inserted.
pub fn insert_fastq_merge_v1(
    conn: &Connection,
    record: &BenchmarkRecord<FastqMergeMetrics>,
) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS bench_fastq_merge_v1 (\
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
        "INSERT INTO bench_fastq_merge_v1 (\
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
            FASTQ_MERGE_SCHEMA_VERSION,
            record.execution.runtime_s,
            record.execution.memory_mb,
            record.execution.exit_code,
            metrics_json,
        ),
    )?;
    Ok(())
}

/// Insert an image QA record into the v1 table.
///
/// # Errors
/// Returns an error if the table cannot be created or the record cannot be inserted.
pub fn insert_image_qa_v1(conn: &Connection, record: &ImageQaRecord) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS image_qa_v1 (\
         tool TEXT NOT NULL,\
         stage TEXT NOT NULL,\
         tool_version TEXT NOT NULL,\
         image_digest TEXT NOT NULL,\
         runner TEXT NOT NULL,\
         platform TEXT NOT NULL,\
         input_hash TEXT NOT NULL,\
         status TEXT NOT NULL,\
         failure_reason TEXT,\
         schema_version INTEGER NOT NULL,\
         outcome_json TEXT NOT NULL\
         )",
        [],
    )?;

    let outcome_json = serde_json::to_string(&record.outcome)?;
    conn.execute(
        "INSERT INTO image_qa_v1 (\
         tool, stage, tool_version, image_digest, runner, platform, input_hash,\
         status, failure_reason, schema_version, outcome_json\
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        (
            &record.tool,
            &record.stage,
            &record.tool_version,
            &record.image_digest,
            &record.runner,
            &record.platform,
            &record.input_hash,
            record.outcome.status(),
            record.outcome.failure_reason(),
            IMAGE_QA_SCHEMA_VERSION,
            outcome_json,
        ),
    )?;
    Ok(())
}

/// Insert an image QA input hash into the v1 table.
///
/// # Errors
/// Returns an error if the table cannot be created or the record cannot be inserted.
pub fn insert_image_qa_input_v1(
    conn: &Connection,
    stage: &str,
    input_hash: &str,
    platform: &str,
    runner: &str,
) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS image_qa_inputs_v1 (\
         stage TEXT NOT NULL,\
         input_hash TEXT NOT NULL,\
         platform TEXT NOT NULL,\
         runner TEXT NOT NULL,\
         schema_version INTEGER NOT NULL,\
         UNIQUE(stage, input_hash, platform, runner)\
         )",
        [],
    )?;
    conn.execute(
        "INSERT OR IGNORE INTO image_qa_inputs_v1 (\
         stage, input_hash, platform, runner, schema_version\
         ) VALUES (?1, ?2, ?3, ?4, ?5)",
        (
            stage,
            input_hash,
            platform,
            runner,
            IMAGE_QA_INPUTS_SCHEMA_VERSION,
        ),
    )?;
    Ok(())
}

/// Load expected QA input hashes for a stage.
///
/// # Errors
/// Returns an error if the query fails.
pub fn image_qa_inputs(
    conn: &Connection,
    stage: &str,
    platform: &str,
    runner: &str,
) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT input_hash FROM image_qa_inputs_v1 \
         WHERE stage = ?1 AND platform = ?2 AND runner = ?3",
    )?;
    let rows = stmt.query_map((stage, platform, runner), |row| row.get(0))?;
    let mut inputs = Vec::new();
    for row in rows {
        inputs.push(row?);
    }
    Ok(inputs)
}

/// Load distinct input hashes from existing image QA records.
///
/// # Errors
/// Returns an error if the query fails.
pub fn image_qa_input_hashes_from_records(
    conn: &Connection,
    stage: &str,
    platform: &str,
    runner: &str,
) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT DISTINCT input_hash FROM image_qa_v1 \
         WHERE stage = ?1 AND platform = ?2 AND runner = ?3",
    )?;
    let rows = stmt.query_map((stage, platform, runner), |row| row.get(0))?;
    let mut inputs = Vec::new();
    for row in rows {
        inputs.push(row?);
    }
    Ok(inputs)
}

/// Check whether image QA passed for a tool/stage/image/platform.
///
/// # Errors
/// Returns an error if the query fails.
pub fn image_qa_passed(
    conn: &Connection,
    tool: &str,
    stage: &str,
    image_digest: &str,
    platform: &str,
    runner: &str,
    input_hash: &str,
) -> Result<bool> {
    let mut stmt = conn.prepare(
        "SELECT COUNT(1) FROM image_qa_v1 \
         WHERE tool = ?1 AND stage = ?2 AND image_digest = ?3 \
         AND platform = ?4 AND runner = ?5 AND input_hash = ?6 AND status = 'pass'",
    )?;
    let count: i64 = stmt.query_row(
        (tool, stage, image_digest, platform, runner, input_hash),
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

#[derive(Debug, Clone)]
pub struct RankingInput<T: StageMetricSchema> {
    pub stage: StageMetricKind,
    pub execution: ExecutionMetrics,
    pub metrics: T,
}

#[must_use]
pub fn normalize_rate(value: f64) -> Option<f64> {
    if value.is_finite() && (0.0..=1.0).contains(&value) {
        Some(value)
    } else {
        None
    }
}

#[must_use]
pub fn normalize_inverse_rate(value: f64) -> Option<f64> {
    normalize_rate(value).map(|v| 1.0 - v)
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

    #[test]
    fn filter_metrics_invariants() -> Result<()> {
        let metrics = FastqFilterMetrics {
            reads_in: 100,
            reads_out: 80,
            reads_dropped: 20,
            mean_q_before: 30.0,
            mean_q_after: 29.0,
        };
        metrics.validate()?;
        let invalid = FastqFilterMetrics {
            reads_in: 100,
            reads_out: 81,
            reads_dropped: 20,
            mean_q_before: 30.0,
            mean_q_after: 31.0,
        };
        assert!(invalid.validate().is_err());
        Ok(())
    }

    #[test]
    fn merge_metrics_invariants() -> Result<()> {
        let metrics = FastqMergeMetrics {
            reads_r1: 100,
            reads_r2: 120,
            reads_merged: 60,
            reads_unmerged: 40,
            merge_rate: 0.6,
        };
        metrics.validate()?;
        let invalid = FastqMergeMetrics {
            reads_r1: 100,
            reads_r2: 100,
            reads_merged: 80,
            reads_unmerged: 30,
            merge_rate: 1.2,
        };
        assert!(invalid.validate().is_err());
        Ok(())
    }
}
