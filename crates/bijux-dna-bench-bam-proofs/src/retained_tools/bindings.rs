use super::observations::{BindingObservation, ExpectedBindingObservation};
use anyhow::{bail, Result};
use std::collections::BTreeSet;
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct BamBindingKey {
    pub(super) stage_id: String,
    pub(super) tool_id: String,
}

pub(super) fn ensure_binding_subset(
    scope: &str,
    surface: &str,
    expected: &BTreeSet<BamBindingKey>,
    actual: &BTreeSet<BamBindingKey>,
) -> Result<()> {
    let expected_stage_ids =
        expected.iter().map(|binding| binding.stage_id.clone()).collect::<BTreeSet<_>>();
    let actual_subset = actual
        .iter()
        .filter(|binding| expected_stage_ids.contains(&binding.stage_id))
        .cloned()
        .collect::<BTreeSet<_>>();
    if actual_subset != *expected {
        bail!(
            "{scope} drifted on {surface}: missing={:?} extra={:?}",
            diff_bindings(expected, &actual_subset),
            diff_bindings(&actual_subset, expected)
        );
    }
    Ok(())
}

pub(super) fn diff_bindings(
    left: &BTreeSet<BamBindingKey>,
    right: &BTreeSet<BamBindingKey>,
) -> Vec<String> {
    left.difference(right)
        .map(|binding| format!("{}/{}", binding.stage_id, binding.tool_id))
        .collect::<Vec<_>>()
}

pub(super) fn binding_key_from_tool_serving_map_row(row: &BindingObservation) -> BamBindingKey {
    BamBindingKey { stage_id: row.stage_id.clone(), tool_id: row.tool_id.clone() }
}

pub(super) fn binding_key_from_command_coverage_row(
    row: &bijux_dna_bench_bam_evidence::command_coverage::BamCommandAdapterCoverageRow,
) -> BamBindingKey {
    BamBindingKey { stage_id: row.stage_id.clone(), tool_id: row.tool_id.clone() }
}

pub(super) fn binding_key_from_parser_fixture_row(
    row: &bijux_dna_bench_bam_evidence::parser_coverage::BamParserFixtureCoverageRow,
) -> BamBindingKey {
    BamBindingKey { stage_id: row.stage_id.clone(), tool_id: row.tool_id.clone() }
}

pub(super) fn binding_key_from_local_smoke_row(row: &BindingObservation) -> BamBindingKey {
    BamBindingKey { stage_id: row.stage_id.clone(), tool_id: row.tool_id.clone() }
}

pub(super) fn binding_key_from_output_declaration_row(
    row: &bijux_dna_bench_bam_evidence::output_contract::BamAdapterOutputContractRow,
) -> BamBindingKey {
    BamBindingKey { stage_id: row.stage_id.clone(), tool_id: row.tool_id.clone() }
}

pub(super) fn binding_key_from_rendered_command_row(row: &BindingObservation) -> BamBindingKey {
    BamBindingKey { stage_id: row.stage_id.clone(), tool_id: row.tool_id.clone() }
}

pub(super) fn binding_key_from_expected_result_row(
    row: &ExpectedBindingObservation,
) -> BamBindingKey {
    BamBindingKey { stage_id: row.stage_id.clone(), tool_id: row.tool_id.clone() }
}

pub(super) fn binding_key_from_report_map_row(
    row: &bijux_dna_bench_bam_evidence::report_map::BamReportMapRow,
) -> BamBindingKey {
    BamBindingKey { stage_id: row.stage_id.clone(), tool_id: row.tool_id.clone() }
}
