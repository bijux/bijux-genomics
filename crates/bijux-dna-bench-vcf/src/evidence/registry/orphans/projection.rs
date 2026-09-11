use super::super::admission::VcfRegistrySnapshot;
use super::model::VcfOrphanToolRow;
use super::policy::ensure_vcf_orphan_tool_contract;
use crate::evidence::serving::VcfToolServingMapReport;
use crate::inputs::VcfAdapterInputs;
use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};
pub(super) fn collect_vcf_orphan_tool_rows(
    inputs: &VcfAdapterInputs,
    registry: &VcfRegistrySnapshot,
    serving_map: &VcfToolServingMapReport,
    required_tool_ids: &BTreeSet<String>,
) -> Result<Vec<VcfOrphanToolRow>> {
    let registry_records = &registry.tool_records;
    let matrix_rows = inputs.bindings();
    let benchmark_stage_ids =
        matrix_rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>();
    let served_stage_count_by_tool =
        serving_map.rows.iter().fold(BTreeMap::<String, usize>::new(), |mut acc, row| {
            *acc.entry(row.tool_id.clone()).or_default() += 1;
            acc
        });

    let mut rows = Vec::new();
    for record in registry_records {
        let served_stage_count =
            served_stage_count_by_tool.get(record.tool_id.as_str()).copied().unwrap_or(0);
        if served_stage_count > 0 {
            continue;
        }

        let has_benchmark_stage_overlap =
            record.stage_ids.iter().any(|stage_id| benchmark_stage_ids.contains(stage_id));
        let decision =
            if required_tool_ids.contains(record.tool_id.as_str()) && has_benchmark_stage_overlap {
                "future_not_benchmark_ready"
            } else {
                "remove_from_scope"
            };

        rows.push(VcfOrphanToolRow {
            tool_id: record.tool_id.clone(),
            registered_binary: record.registered_binary.clone(),
            served_stage_count,
            decision: decision.to_string(),
        });
    }

    rows.sort_by(|left, right| left.tool_id.cmp(&right.tool_id));
    ensure_vcf_orphan_tool_contract(&rows, required_tool_ids)?;
    Ok(rows)
}
