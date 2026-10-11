use super::declarations::TRIM_STAGE_SPECS;
use super::models::TrimStageSpec;
use anyhow::anyhow;
use anyhow::Result;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
pub(super) fn ensure_trim_stage_tool_sets<'a>(
    label: &str,
    pairs: impl Iterator<Item = (&'a str, &'a str)>,
) -> Result<()> {
    let mut observed = BTreeMap::<String, BTreeSet<String>>::new();
    for (stage_id, tool_id) in pairs {
        observed.entry(stage_id.to_string()).or_default().insert(tool_id.to_string());
    }

    for spec in TRIM_STAGE_SPECS {
        let actual = observed.get(spec.stage_id).cloned().unwrap_or_default();
        let expected = spec
            .expected_tool_ids
            .iter()
            .map(|tool_id| (*tool_id).to_string())
            .collect::<BTreeSet<_>>();
        if actual != expected {
            return Err(anyhow!(
                "{} for `{}` must match the governed trim tool set: expected [{}], found [{}]",
                label,
                spec.stage_id,
                expected.iter().cloned().collect::<Vec<_>>().join(", "),
                actual.iter().cloned().collect::<Vec<_>>().join(", ")
            ));
        }
    }
    Ok(())
}

pub(super) fn ensure_required_stage_rows<'a>(
    label: &str,
    stage_ids: impl Iterator<Item = &'a str>,
) -> Result<()> {
    let observed = stage_ids.map(str::to_string).collect::<BTreeSet<_>>();
    let expected =
        TRIM_STAGE_SPECS.iter().map(|spec| spec.stage_id.to_string()).collect::<BTreeSet<_>>();
    if observed != expected {
        return Err(anyhow!(
            "{} must cover [{}], found [{}]",
            label,
            expected.iter().cloned().collect::<Vec<_>>().join(", "),
            observed.iter().cloned().collect::<Vec<_>>().join(", ")
        ));
    }
    Ok(())
}

pub(super) fn trim_stage_spec(stage_id: &str) -> Option<TrimStageSpec> {
    TRIM_STAGE_SPECS.iter().copied().find(|spec| spec.stage_id == stage_id)
}

pub(super) fn trim_stage_binding_admitted(stage_id: &str, tool_id: &str) -> bool {
    trim_stage_spec(stage_id).is_some_and(|spec| spec.expected_tool_ids.contains(&tool_id))
}

pub(super) fn has_governed_support(value: &str) -> bool {
    matches!(
        value,
        "governed_execution" | "governed_benchmark_cohort" | "observer_specialized_benchmark"
    )
}

pub(super) fn has_runnable_adapter(value: &str) -> bool {
    matches!(value, "runnable" | "plannable")
}

pub(super) fn has_fixture_corpus(value: &str) -> bool {
    value.starts_with("fixture:")
}
