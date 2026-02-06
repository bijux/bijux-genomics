//! Owner: bijux-analyze
//! Row-to-struct mapping helpers for `SQLite` benchmark storage.

use serde::de::DeserializeOwned;
use serde_json::Value as JsonValue;

use bijux_core::foundation::measure::ExecutionMetrics;
use bijux_core::metrics::MetricSet;

use crate::aggregate::{BenchmarkContext, BenchmarkRecord, StageMetricSchema};
use crate::model::JsonBlob;

use super::json_from_str;

pub(super) fn benchmark_record_from_row<T>(
    row: &rusqlite::Row<'_>,
) -> std::result::Result<BenchmarkRecord<T>, rusqlite::Error>
where
    T: DeserializeOwned + StageMetricSchema,
{
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
    let metrics: MetricSet<T> = json_from_str(&metrics_json)?;
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
}
