use super::super::adapters::{FastqBenchmarkStatus, FastqCommandAdapterCoverageRow};
use super::super::inputs::FastqScopeRow;
use super::contract::ensure_fastq_active_stage_tool_matrix_contract;
use super::model::{
    BindingKey, FastqActiveStageToolMatrixCollection, FastqActiveStageToolMatrixRow,
};
use super::{
    ACTIVE_SCOPE_PROOF_PATH, BENCHMARK_READY_STATUS, DECLARED_ONLY_ADAPTER_STATUS,
    PLANNED_SUPPORT_STATUS,
};
use anyhow::{anyhow, Result};
use std::collections::{BTreeMap, BTreeSet};

/// Evaluate admitted FASTQ readiness inputs.
///
/// # Errors
/// Rejects invalid identities, incompatible evidence or failed publication.
pub fn collect_fastq_active_stage_tool_matrix_rows(
    coverage: super::super::inputs::FastqAdapterCoverage,
    active_rows: Vec<super::super::inputs::FastqScopeRow>,
) -> Result<FastqActiveStageToolMatrixCollection> {
    let (retained_stage_count, retained_tool_count, coverage_rows) = coverage;
    super::super::inputs::admit_rows(
        retained_stage_count,
        retained_tool_count,
        coverage_rows.iter().map(|row| (row.stage_id.as_str(), row.tool_id.as_str())),
    )?;
    let coverage_by_binding = coverage_rows
        .iter()
        .map(|row| {
            (
                BindingKey { stage_id: row.stage_id.clone(), tool_id: row.tool_id.clone() },
                row.clone(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let removed_rows = coverage_rows
        .iter()
        .filter(|row| row.benchmark_status != FastqBenchmarkStatus::BenchmarkReady)
        .cloned()
        .collect::<Vec<_>>();

    let mut rows = active_rows
        .into_iter()
        .filter(|row| row.domain == "fastq")
        .map(|active_row| {
            let key = BindingKey {
                stage_id: active_row.stage_id.clone(),
                tool_id: active_row.tool_id.clone(),
            };
            let coverage_row = coverage_by_binding.get(&key).ok_or_else(|| {
                anyhow!(
                    "FASTQ active-stage-tool matrix is missing command-adapter coverage for `{}` / `{}`",
                    active_row.stage_id,
                    active_row.tool_id
                )
            })?;
            build_fastq_active_stage_tool_matrix_row(active_row, coverage_row)
        })
        .collect::<Result<Vec<_>>>()?;

    rows.sort_by(|left, right| {
        left.stage_id.cmp(&right.stage_id).then_with(|| left.tool_id.cmp(&right.tool_id))
    });
    ensure_fastq_active_stage_tool_matrix_contract(&rows)?;

    Ok(FastqActiveStageToolMatrixCollection {
        retained_row_count: coverage_rows.len(),
        retained_stage_count,
        retained_tool_count,
        removed_row_count: removed_rows.len(),
        removed_stage_count: removed_rows
            .iter()
            .map(|row| row.stage_id.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        removed_tool_count: removed_rows
            .iter()
            .map(|row| row.tool_id.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        rows,
    })
}

pub(super) fn build_fastq_active_stage_tool_matrix_row(
    active_row: FastqScopeRow,
    coverage_row: &FastqCommandAdapterCoverageRow,
) -> Result<FastqActiveStageToolMatrixRow> {
    if coverage_row.benchmark_status != FastqBenchmarkStatus::BenchmarkReady {
        return Err(anyhow!(
            "FASTQ active-stage-tool matrix cannot retain non-benchmark-ready row `{}` / `{}`",
            active_row.stage_id,
            active_row.tool_id
        ));
    }
    if coverage_row.support_status == PLANNED_SUPPORT_STATUS {
        return Err(anyhow!(
            "FASTQ active-stage-tool matrix cannot retain planned support row `{}` / `{}`",
            active_row.stage_id,
            active_row.tool_id
        ));
    }
    if coverage_row.adapter_status == DECLARED_ONLY_ADAPTER_STATUS {
        return Err(anyhow!(
            "FASTQ active-stage-tool matrix cannot retain declared-only adapter row `{}` / `{}`",
            active_row.stage_id,
            active_row.tool_id
        ));
    }
    if coverage_row.corpus_status == "planner_only" {
        return Err(anyhow!(
            "FASTQ active-stage-tool matrix cannot retain excluded corpus row `{}` / `{}`",
            active_row.stage_id,
            active_row.tool_id
        ));
    }

    Ok(FastqActiveStageToolMatrixRow {
        stage_id: active_row.stage_id.clone(),
        tool_id: active_row.tool_id.clone(),
        corpus_id: active_row.corpus_id,
        asset_profile_id: active_row.asset_profile_id,
        adapter_id: active_row.adapter_id,
        parser_id: active_row.parser_id,
        schema_id: active_row.schema_id,
        benchmark_status: BENCHMARK_READY_STATUS.to_string(),
        support_status: coverage_row.support_status.clone(),
        adapter_status: coverage_row.adapter_status.clone(),
        parser_status: coverage_row.parser_status.clone(),
        corpus_status: coverage_row.corpus_status.clone(),
        scope_proof_path: ACTIVE_SCOPE_PROOF_PATH.to_string(),
        reason: format!(
            "binding `{}` / `{}` remains in governed FASTQ active scope because it is benchmark_ready with executable adapter coverage, normalized parser coverage, and governed benchmark-scope coverage",
            active_row.stage_id, active_row.tool_id
        ),
    })
}
