use crate::inputs::VcfAdapterInputs;
use anyhow::{ensure, Result};
use bijux_dna_bench_vcf_evidence::model::scope::VcfActiveStageToolMatrixRow;
use std::collections::{BTreeMap, BTreeSet};
pub(super) fn admit_retained(
    inputs: &VcfAdapterInputs,
    rows: &[VcfActiveStageToolMatrixRow],
) -> Result<()> {
    crate::evidence::serving::collect_vcf_tool_serving_map_rows(inputs)?;
    let bindings = inputs
        .bindings()
        .iter()
        .map(|row| {
            (
                (
                    row.stage_id.as_str(),
                    row.tool_id.as_str(),
                    row.corpus_id.as_str(),
                    row.asset_profile_id.as_str(),
                ),
                row,
            )
        })
        .collect::<BTreeMap<_, _>>();
    let schemas = bijux_dna_bench_model::normalized_result::vcf_envelope::vcf_normalized_metrics_stage_descriptors()?
        .into_iter()
        .map(|row| (row.stage_id, row.schema_id))
        .collect::<BTreeMap<_, _>>();
    let mut seen = BTreeSet::new();
    for row in rows {
        let key = (
            row.stage_id.as_str(),
            row.tool_id.as_str(),
            row.corpus_id.as_str(),
            row.asset_profile_id.as_str(),
        );
        ensure!(seen.insert(key), "parser fixture coverage repeats a retained binding");
        let binding = bindings.get(&key).ok_or_else(|| {
            anyhow::anyhow!("parser fixture coverage contains a foreign retained binding")
        })?;
        ensure!(
            row.adapter_id == binding.adapter_id
                && row.parser_id == binding.parser_id
                && schemas.get(&row.stage_id) == Some(&row.schema_id),
            "parser fixture coverage retained metadata disagrees with admitted inputs"
        );
        ensure!(
            matches!(row.scope_state.as_str(), "active" | "complete" | "removed_from_scope"),
            "parser fixture coverage has unknown retained scope state"
        );
    }
    Ok(())
}
