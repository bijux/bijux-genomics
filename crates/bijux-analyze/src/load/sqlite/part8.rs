//! Owner: bijux-analyze
//! `SQLite` benchmark helpers part 8.

use rusqlite::{params, Connection};
use serde_json::Value as JsonValue;

use bijux_core::measure::ExecutionMetrics;
use bijux_core::metrics::MetricSet;

use crate::aggregate::{BenchmarkContext, BenchmarkRecord, FastqStatsMetrics, Result};
use crate::model::JsonBlob;

use super::base::json_from_str;

/// Load a stats benchmark record from `SQLite` if present.
///
/// # Errors
/// Returns an error if the query or JSON parsing fails.
#[allow(clippy::too_many_arguments)]
pub fn fetch_fastq_stats_v1(
    conn: &Connection,
    tool: &str,
    tool_version: &str,
    image_digest: &str,
    runner: &str,
    platform: &str,
    input_hash: &str,
    params_hash: &str,
) -> Result<Option<BenchmarkRecord<FastqStatsMetrics>>> {
    let mut stmt = conn.prepare(
        "SELECT tool, tool_version, image_digest, runner, platform, input_hash, params_hash,\
         parameters_json, runtime_s, memory_mb, exit_code, metrics_json \
         FROM bench_fastq_stats_v1 \
         WHERE tool = ?1 AND tool_version = ?2 AND image_digest = ?3\
         AND runner = ?4 AND platform = ?5 AND input_hash = ?6 AND params_hash = ?7\
         ORDER BY record_id DESC, inserted_at DESC LIMIT 1",
    )?;
    let row = stmt.query_row(
        params![
            tool,
            tool_version,
            image_digest,
            runner,
            platform,
            input_hash,
            params_hash
        ],
        |row| {
            let tool: String = row.get(0)?;
            let tool_version: String = row.get(1)?;
            let image_digest: String = row.get(2)?;
            let runner: String = row.get(3)?;
            let platform: String = row.get(4)?;
            let input_hash: String = row.get(5)?;
            let _params_hash: String = row.get(6)?;
            let parameters_json: String = row.get(7)?;
            let runtime_s: f64 = row.get(8)?;
            let memory_mb: f64 = row.get(9)?;
            let exit_code: i64 = row.get(10)?;
            let metrics_json: String = row.get(11)?;
            let parameters: JsonValue = json_from_str(&parameters_json)?;
            let metrics: MetricSet<FastqStatsMetrics> = json_from_str(&metrics_json)?;
            Ok(BenchmarkRecord {
                context: BenchmarkContext {
                    tool,
                    tool_version,
                    image_digest,
                    runner,
                    platform,
                    input_hash,
                    parameters: JsonBlob::from(parameters),
                },
                execution: ExecutionMetrics {
                    runtime_s,
                    memory_mb,
                    exit_code: i32::try_from(exit_code).unwrap_or(i32::MAX),
                },
                metrics,
            })
        },
    );
    match row {
        Ok(record) => Ok(Some(record)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(err) => Err(err.into()),
    }
}
