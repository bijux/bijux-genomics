use super::super::admission::VcfRegistrySnapshot;
use super::model::{VcfMatrixRegistryConsistencyReport, VcfMatrixRegistryConsistencyRow};
use super::policy::sorted_status_label;
use super::VCF_MATRIX_REGISTRY_CONSISTENCY_SCHEMA_VERSION;
use crate::inputs::VcfAdapterInputs;
use anyhow::{anyhow, Result};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
pub(super) fn build_report(
    inputs: &VcfAdapterInputs,
    registry: &VcfRegistrySnapshot,
    repo_root: &Path,
    output_path: &Path,
) -> Result<VcfMatrixRegistryConsistencyReport> {
    let stage_support_by_id = inputs
        .catalog()
        .iter()
        .map(|row| (row.stage_id.clone(), row.support_status.clone()))
        .collect::<BTreeMap<_, _>>();
    let matrix_rows = inputs.bindings();
    let registry_records = &registry.tool_records;
    let registry_pair_count = registry_records.iter().map(|record| record.stage_ids.len()).sum();
    let registry_by_tool = registry_records
        .iter()
        .cloned()
        .map(|record| (record.tool_id.clone(), record))
        .collect::<BTreeMap<_, _>>();
    let registry_pairs = registry_records
        .iter()
        .flat_map(|record| {
            record
                .stage_ids
                .iter()
                .cloned()
                .map(|stage_id| (stage_id, record.tool_id.clone()))
                .collect::<Vec<_>>()
        })
        .collect::<BTreeSet<_>>();

    let matrix_pairs = matrix_rows
        .iter()
        .map(|row| (row.stage_id.clone(), row.tool_id.clone()))
        .collect::<BTreeSet<_>>();

    let benchmark_ready_registry_pairs = registry_records
        .iter()
        .filter(|record| {
            record.statuses.contains("production") || record.statuses.contains("supported")
        })
        .flat_map(|record| {
            record
                .stage_ids
                .iter()
                .filter(|stage_id| {
                    stage_support_by_id
                        .get(stage_id.as_str())
                        .is_some_and(|status| status == "supported")
                })
                .cloned()
                .map(|stage_id| (stage_id, record.tool_id.clone()))
                .collect::<Vec<_>>()
        })
        .collect::<BTreeSet<_>>();

    let mut rows = Vec::new();
    append_unregistered_rows(
        &mut rows,
        &matrix_pairs,
        &registry_pairs,
        &registry_by_tool,
        &stage_support_by_id,
    );

    append_missing_matrix_rows(
        &mut rows,
        &benchmark_ready_registry_pairs,
        &matrix_pairs,
        &registry_by_tool,
        &stage_support_by_id,
    )?;

    rows.sort_by(|left, right| {
        left.stage_id
            .cmp(&right.stage_id)
            .then_with(|| left.tool_id.cmp(&right.tool_id))
            .then_with(|| left.drift_kind.cmp(&right.drift_kind))
    });

    let report = VcfMatrixRegistryConsistencyReport {
        schema_version: VCF_MATRIX_REGISTRY_CONSISTENCY_SCHEMA_VERSION,
        domain: "vcf",
        output_path: path_relative_to_repo(repo_root, output_path),
        passes_gate: rows.is_empty(),
        stage_count: stage_support_by_id.len(),
        matrix_row_count: matrix_rows.len(),
        registry_pair_count,
        benchmark_ready_registry_pair_count: benchmark_ready_registry_pairs.len(),
        unregistered_matrix_pair_count: rows
            .iter()
            .filter(|row| row.drift_kind == "matrix_row_unregistered")
            .count(),
        missing_benchmark_ready_registry_pair_count: rows
            .iter()
            .filter(|row| row.drift_kind == "benchmark_ready_registry_pair_missing_from_matrix")
            .count(),
        rows,
    };
    Ok(report)
}
fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

fn append_missing_matrix_rows(
    rows: &mut Vec<VcfMatrixRegistryConsistencyRow>,
    benchmark_ready_registry_pairs: &BTreeSet<(String, String)>,
    matrix_pairs: &BTreeSet<(String, String)>,
    registry_by_tool: &BTreeMap<String, super::super::admission::RegistryToolRecord>,
    stage_support_by_id: &BTreeMap<String, String>,
) -> Result<()> {
    for (stage_id, tool_id) in benchmark_ready_registry_pairs.difference(matrix_pairs) {
        let registry_record = registry_by_tool
            .get(tool_id.as_str())
            .ok_or_else(|| anyhow!("missing registry record for `{tool_id}`"))?;
        let registry_stage_ids = registry_record.stage_ids.iter().cloned().collect::<Vec<_>>();
        rows.push(VcfMatrixRegistryConsistencyRow {
            stage_id: stage_id.clone(),
            tool_id: tool_id.clone(),
            drift_kind: "benchmark_ready_registry_pair_missing_from_matrix".to_string(),
            stage_support_status: stage_support_by_id
                .get(stage_id.as_str())
                .cloned()
                .unwrap_or_else(|| "unknown".to_string()),
            tool_registry_status: sorted_status_label(&registry_record.statuses),
            registry_stage_ids: registry_stage_ids.clone(),
            reason: format!(
                "VCF registry marks `{stage_id}` / `{tool_id}` as benchmark-ready for the governed production slice, but the VCF matrix does not benchmark it; registered stages: {}",
                registry_stage_ids.join(", ")
            ),
        });
    }
    Ok(())
}

fn append_unregistered_rows(
    rows: &mut Vec<VcfMatrixRegistryConsistencyRow>,
    matrix_pairs: &BTreeSet<(String, String)>,
    registry_pairs: &BTreeSet<(String, String)>,
    registry_by_tool: &BTreeMap<String, super::super::admission::RegistryToolRecord>,
    stage_support_by_id: &BTreeMap<String, String>,
) {
    for (stage_id, tool_id) in matrix_pairs.difference(registry_pairs) {
        let registry_record = registry_by_tool.get(tool_id.as_str());
        let stage_support_status = stage_support_by_id
            .get(stage_id.as_str())
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        let tool_registry_status = registry_record.map_or_else(
            || "tool_missing".to_string(),
            |record| sorted_status_label(&record.statuses),
        );
        let registry_stage_ids = registry_record
            .map(|record| record.stage_ids.iter().cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        rows.push(VcfMatrixRegistryConsistencyRow {
            stage_id: stage_id.clone(),
            tool_id: tool_id.clone(),
            drift_kind: "matrix_row_unregistered".to_string(),
            stage_support_status,
            tool_registry_status: tool_registry_status.clone(),
            registry_stage_ids: registry_stage_ids.clone(),
            reason: format!(
                "VCF matrix row `{stage_id}` / `{tool_id}` is benchmarked but the VCF registry does not admit that pair; tool registry status: {tool_registry_status}; registered stages: {}",
                if registry_stage_ids.is_empty() {
                    "<none>".to_string()
                } else {
                    registry_stage_ids.join(", ")
                }
            ),
        });
    }
}
