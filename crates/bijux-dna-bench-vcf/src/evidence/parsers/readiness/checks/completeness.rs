use super::super::state::ReadinessState;
use anyhow::{anyhow, bail, Result};
use std::collections::BTreeSet;
pub(super) fn complete_pairs(state: &ReadinessState) -> Result<String> {
    let parser_pairs = state
        .parser_coverage_report
        .as_ref()
        .ok_or_else(|| anyhow!("goal 256 parser coverage report was not produced"))?
        .rows
        .iter()
        .map(|row| (row.stage_id.clone(), row.tool_id.clone()))
        .collect::<BTreeSet<_>>();
    let expected_pairs = state
        .expected_results_report
        .as_ref()
        .ok_or_else(|| anyhow!("goal 257 expected benchmark results report was not produced"))?
        .rows
        .iter()
        .map(|row| (row.stage_id.clone(), row.tool_id.clone()))
        .collect::<BTreeSet<_>>();
    let report_pairs = state
        .report_map_report
        .as_ref()
        .ok_or_else(|| anyhow!("goal 259 report map was not produced"))?
        .rows
        .iter()
        .map(|row| (row.stage_id.clone(), row.tool_id.clone()))
        .collect::<BTreeSet<_>>();

    ensure_pair_sets_match("parser coverage", &expected_pairs, &parser_pairs)?;
    ensure_pair_sets_match("report map", &expected_pairs, &report_pairs)?;

    Ok(format!(
                "validated {} benchmark-ready VCF rows across parser coverage, expected results, and report-map coverage",
                expected_pairs.len()
            ))
}

pub(in super::super) fn ensure_pair_sets_match(
    surface: &str,
    expected: &BTreeSet<(String, String)>,
    observed: &BTreeSet<(String, String)>,
) -> Result<()> {
    let missing = expected.difference(observed).cloned().collect::<Vec<_>>();
    let extra = observed.difference(expected).cloned().collect::<Vec<_>>();
    if missing.is_empty() && extra.is_empty() {
        return Ok(());
    }

    let format_pairs = |pairs: &[(String, String)]| {
        if pairs.is_empty() {
            "none".to_string()
        } else {
            pairs
                .iter()
                .map(|(stage_id, tool_id)| format!("{stage_id}:{tool_id}"))
                .collect::<Vec<_>>()
                .join(", ")
        }
    };

    bail!(
        "{surface} drifted from the canonical benchmark-ready VCF pair slice: missing [{}], extra [{}]",
        format_pairs(&missing),
        format_pairs(&extra)
    );
}
