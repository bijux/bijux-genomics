//! Owner: repository observation capture for one micro-benchmark report.
mod declarations;
mod insufficiency;
pub(crate) mod publication;
mod resources;
use anyhow::{Context, Result};
use bijux_dna_bench_model::{
    MicroBenchmarkMemorySourceRow, MicroBenchmarkReportObservations, MicroBenchmarkResultRow,
    MicroBenchmarkRuntimeRow,
};
use declarations::{load_science_threshold_rows, load_stage_tool_resources};
use insufficiency::detect_insufficient_data_row;
use resources::{load_memory_evidence, load_runtime_evidence};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
const SCIENTIFIC_ACCEPTANCE_THRESHOLDS_PATH: &str =
    "configs/bench/local/scientific-acceptance-thresholds.toml";
const STAGE_TOOL_RESOURCES_PATH: &str = "configs/bench/local/stage-tool-resources.toml";
pub(crate) fn capture(
    repo_root: &Path,
    result_rows: &[MicroBenchmarkResultRow],
    governed_runtime_projection: bool,
) -> Result<MicroBenchmarkReportObservations> {
    let insufficient_data_rows = result_rows
        .iter()
        .filter_map(|row| detect_insufficient_data_row(repo_root, row).transpose())
        .collect::<Result<Vec<_>>>()?;

    let stage_tool_resources =
        load_stage_tool_resources(repo_root.join(STAGE_TOOL_RESOURCES_PATH))?;
    let science_threshold_rows = load_science_threshold_rows(
        repo_root.join(SCIENTIFIC_ACCEPTANCE_THRESHOLDS_PATH),
        result_rows,
    )?;

    let mut runtime_rows = Vec::with_capacity(result_rows.len());
    let mut memory_source_rows = Vec::with_capacity(result_rows.len());
    for row in result_rows {
        let runtime = load_runtime_evidence(repo_root, row, governed_runtime_projection)?;
        let memory = load_memory_evidence(repo_root, row, &stage_tool_resources)?;
        runtime_rows.push(MicroBenchmarkRuntimeRow {
            execution_id: row.execution_id.clone(),
            component_id: row.component_id.clone(),
            domain: row.domain.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            execution_status: row.status,
            elapsed_seconds: runtime.elapsed_seconds,
            runtime_source: runtime.runtime_source.to_string(),
        });
        memory_source_rows.push(MicroBenchmarkMemorySourceRow {
            execution_id: row.execution_id.clone(),
            component_id: row.component_id.clone(),
            domain: row.domain.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            execution_status: row.status,
            declared_memory_mb: memory.declared_memory_mb,
            declared_cpu_threads: memory.declared_cpu_threads,
            observed_memory_mb: memory.observed_memory_mb,
            observed_cpu_threads: memory.observed_cpu_threads,
            memory_source: memory.memory_source.to_string(),
            resource_origin: memory.resource_origin,
        });
    }

    Ok(MicroBenchmarkReportObservations {
        existing_evidence_paths: result_rows
            .iter()
            .filter_map(|row| row.evidence_path.as_ref())
            .filter(|path| repo_root.join(path).is_file())
            .cloned()
            .collect(),
        insufficient: insufficient_data_rows,
        runtime: runtime_rows,
        memory: memory_source_rows,
        thresholds: science_threshold_rows,
    })
}
fn read_json_value(path: PathBuf) -> Result<Value> {
    let payload = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&payload).with_context(|| format!("parse {}", path.display()))
}
