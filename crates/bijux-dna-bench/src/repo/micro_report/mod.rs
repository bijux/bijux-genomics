//! Owner: repository observation capture for one micro-benchmark report.
mod acquisition;
mod declarations;
mod insufficiency;
pub(crate) mod publication;
mod resources;
use acquisition::ResultEvidence;
use anyhow::Result;
use bijux_dna_bench_model::{
    MicroBenchmarkMemorySourceRow, MicroBenchmarkReportObservations, MicroBenchmarkResultRow,
    MicroBenchmarkRuntimeRow,
};
use declarations::{load_science_threshold_rows, load_stage_tool_resources};
use insufficiency::detect_insufficient_data_row;
use resources::{load_memory_evidence, load_runtime_evidence};
use std::path::Path;
const SCIENTIFIC_ACCEPTANCE_THRESHOLDS_PATH: &str =
    "configs/bench/local/scientific-acceptance-thresholds.toml";
const STAGE_TOOL_RESOURCES_PATH: &str = "configs/bench/local/stage-tool-resources.toml";
pub(crate) fn capture(
    repo_root: &Path,
    result_rows: &[MicroBenchmarkResultRow],
    governed_runtime_projection: bool,
) -> Result<MicroBenchmarkReportObservations> {
    let mut insufficient_data_rows = Vec::new();
    let stage_tool_resources =
        load_stage_tool_resources(repo_root.join(STAGE_TOOL_RESOURCES_PATH))?;
    let science_threshold_rows = load_science_threshold_rows(
        repo_root.join(SCIENTIFIC_ACCEPTANCE_THRESHOLDS_PATH),
        result_rows,
    )?;

    let mut runtime_rows = Vec::with_capacity(result_rows.len());
    let mut memory_source_rows = Vec::with_capacity(result_rows.len());
    for row in result_rows {
        let mut evidence = ResultEvidence::new(
            repo_root,
            row.evidence_path.as_deref(),
            row.stage_result_manifest_path.as_deref(),
        );
        if let Some(insufficient) = detect_insufficient_data_row(&mut evidence, row)? {
            insufficient_data_rows.push(insufficient);
        }
        let runtime = load_runtime_evidence(&mut evidence, row, governed_runtime_projection)?;
        let memory = load_memory_evidence(&mut evidence, row, &stage_tool_resources)?;
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
