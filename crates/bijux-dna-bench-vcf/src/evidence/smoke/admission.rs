use super::VcfSmokeInputs;
use anyhow::{ensure, Result};
use std::collections::BTreeSet;
pub(super) fn admit_scope(inputs: &VcfSmokeInputs<'_>) -> Result<()> {
    let declared = inputs
        .bindings
        .bindings()
        .iter()
        .map(|row| (&row.stage_id, &row.tool_id, &row.corpus_id, &row.asset_profile_id))
        .collect::<BTreeSet<_>>();
    let mut retained = BTreeSet::new();
    for row in inputs.retained {
        ensure!(
            declared.contains(&(
                &row.stage_id,
                &row.tool_id,
                &row.corpus_id,
                &row.asset_profile_id
            )),
            "VCF smoke scope contains a foreign retained binding"
        );
        ensure!(
            retained.insert((&row.stage_id, &row.tool_id)),
            "VCF smoke scope repeats a retained stage/tool binding"
        );
        ensure!(
            matches!(row.scope_state.as_str(), "active" | "complete" | "removed_from_scope"),
            "VCF smoke scope has an unknown state"
        );
    }
    ensure!(
        retained.len() == declared.len(),
        "VCF smoke scope must cover every admitted retained binding"
    );
    for (stage, tool) in inputs.stage_defaults {
        ensure!(
            retained.contains(&(stage, tool)),
            "VCF smoke default does not identify a retained binding"
        );
    }
    ensure!(
        inputs.stage_defaults.len()
            == inputs.retained.iter().map(|row| &row.stage_id).collect::<BTreeSet<_>>().len(),
        "VCF smoke defaults must cover every retained stage"
    );
    ensure!(
        inputs.external_tool_ids.iter().all(|id| !id.is_empty() && id.trim() == id),
        "VCF external delivery identities must be nonempty and unpadded"
    );
    Ok(())
}
