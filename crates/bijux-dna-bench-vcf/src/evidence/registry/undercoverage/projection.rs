use super::super::admission::VcfRegistrySnapshot;
use super::model::VcfUndercoveredStageRow;
use super::policy::{
    classify_undercoverage_decision, ensure_vcf_undercovered_stage_contract, vcf_tool_class_label,
};
use crate::evidence::serving::VcfToolServingMapReport;
use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};
pub(super) fn collect_vcf_undercovered_stage_rows(
    registry: &VcfRegistrySnapshot,
    serving_map: &VcfToolServingMapReport,
) -> Result<Vec<VcfUndercoveredStageRow>> {
    let registry_records = &registry.tool_records;
    let registry_by_tool = registry_records
        .iter()
        .cloned()
        .map(|record| (record.tool_id.clone(), record))
        .collect::<BTreeMap<_, _>>();
    let mut valid_tools_by_stage = BTreeMap::<String, BTreeSet<String>>::new();
    for record in registry_records {
        for stage_id in &record.stage_ids {
            valid_tools_by_stage
                .entry(stage_id.clone())
                .or_default()
                .insert(record.tool_id.clone());
        }
    }

    let mut registered_tools_by_stage = BTreeMap::<String, BTreeSet<String>>::new();
    for row in &serving_map.rows {
        registered_tools_by_stage
            .entry(row.stage_id.clone())
            .or_default()
            .insert(row.tool_id.clone());
    }

    let mut rows = Vec::new();
    for (stage_id, valid_tools) in valid_tools_by_stage {
        let registered_tools = registered_tools_by_stage
            .get(stage_id.as_str())
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .collect::<Vec<_>>();
        if valid_tools.len() <= 1
            || registered_tools.is_empty()
            || registered_tools.len() >= valid_tools.len()
        {
            continue;
        }

        let missing_tools = valid_tools
            .iter()
            .filter(|tool_id| !registered_tools.contains(tool_id))
            .cloned()
            .collect::<Vec<_>>();
        if missing_tools.is_empty() {
            continue;
        }

        let valid_tool_classes = valid_tools
            .iter()
            .map(|tool_id| vcf_tool_class_label(tool_id))
            .collect::<Result<BTreeSet<_>>>()?
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>();
        let decision =
            classify_undercoverage_decision(&stage_id, &missing_tools, &registry_by_tool)?;

        rows.push(VcfUndercoveredStageRow {
            stage_id,
            valid_tool_classes,
            registered_tools,
            missing_tools,
            decision: decision.to_string(),
        });
    }

    rows.sort_by(|left, right| left.stage_id.cmp(&right.stage_id));
    ensure_vcf_undercovered_stage_contract(&rows)?;
    Ok(rows)
}
