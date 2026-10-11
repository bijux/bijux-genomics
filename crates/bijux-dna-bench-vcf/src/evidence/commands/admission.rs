use crate::inputs::VcfAdapterInputs;
use anyhow::{ensure, Result};
use bijux_dna_bench_vcf_evidence::model::scope::VcfActiveStageToolMatrixRow;
use std::collections::BTreeSet;
pub(super) fn admit_retained(
    inputs: &VcfAdapterInputs,
    retained: &[VcfActiveStageToolMatrixRow],
) -> Result<()> {
    let schemas=bijux_dna_bench_model::normalized_result::vcf_envelope::vcf_normalized_metrics_stage_descriptors()?;
    let mut keys = BTreeSet::new();
    for row in retained {
        ensure!(
            matches!(row.scope_state.as_str(), "active" | "complete" | "removed_from_scope"),
            "VCF command scope has an unknown state"
        );
        ensure!(
            [
                &row.stage_id,
                &row.tool_id,
                &row.corpus_id,
                &row.asset_profile_id,
                &row.adapter_id,
                &row.parser_id,
                &row.schema_id
            ]
            .into_iter()
            .all(|v| !v.is_empty() && v.trim() == v),
            "VCF command scope identities must be nonempty and unpadded"
        );
        ensure!(
            keys.insert((&row.stage_id, &row.tool_id)),
            "VCF command scope repeats a stage/tool authority"
        );
        if row.scope_state != "active" {
            continue;
        }
        let declarations = inputs
            .bindings()
            .iter()
            .filter(|candidate| {
                candidate.stage_id == row.stage_id && candidate.tool_id == row.tool_id
            })
            .collect::<Vec<_>>();
        ensure!(
            declarations.len() == 1,
            "VCF commands require exactly one input binding per active stage/tool"
        );
        let declaration = declarations[0];
        ensure!(
            row.corpus_id == declaration.corpus_id
                && row.asset_profile_id == declaration.asset_profile_id
                && row.adapter_id == declaration.adapter_id
                && row.parser_id == declaration.parser_id
                && schemas
                    .iter()
                    .any(|schema| schema.stage_id == row.stage_id
                        && schema.schema_id == row.schema_id),
            "VCF command scope disagrees with admitted input identities"
        );
    }
    Ok(())
}
