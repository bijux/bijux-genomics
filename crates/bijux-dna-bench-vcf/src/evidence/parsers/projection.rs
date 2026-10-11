use crate::inputs::VcfAdapterInputs;
use anyhow::Result;
use bijux_dna_bench_vcf_evidence::model::{
    parsers::{VcfParserFixtureCoverageRow, VcfParserFixtureCoverageStatus},
    scope::VcfActiveStageToolMatrixRow,
};
use bijux_dna_domain_vcf::{
    contracts::stage_metrics_contract, find_vcf_parser_fixture_inventory_row, VcfDomainStage,
};
use std::{collections::BTreeSet, path::Path};
/// Observe fixture coverage for explicitly admitted retained bindings.
///
/// # Errors
/// Returns retained identity, fixture acquisition or stage contract errors.
pub fn collect_vcf_parser_fixture_coverage_rows_from_retained(
    inputs: &VcfAdapterInputs,
    repo_root: &Path,
    retained: &[VcfActiveStageToolMatrixRow],
) -> Result<(usize, usize, Vec<VcfParserFixtureCoverageRow>)> {
    super::admission::admit_retained(inputs, retained)?;
    let active = retained.iter().filter(|row| row.scope_state == "active").collect::<Vec<_>>();
    let stage_count = active.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>().len();
    let tool_count = active.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>().len();
    let mut rows = active
        .into_iter()
        .map(|row| observe_active_row(repo_root, row))
        .collect::<Result<Vec<_>>>()?;
    rows.sort_by(|left, right| {
        left.stage_id.cmp(&right.stage_id).then_with(|| left.tool_id.cmp(&right.tool_id))
    });
    Ok((stage_count, tool_count, rows))
}
fn observe_active_row(
    repo_root: &Path,
    active: &VcfActiveStageToolMatrixRow,
) -> Result<VcfParserFixtureCoverageRow> {
    let stage = VcfDomainStage::try_from(active.stage_id.as_str())?;
    let mut row = VcfParserFixtureCoverageRow {
        stage_id: active.stage_id.clone(),
        tool_id: active.tool_id.clone(),
        corpus_id: active.corpus_id.clone(),
        asset_profile_id: active.asset_profile_id.clone(),
        adapter_id: active.adapter_id.clone(),
        parser_id: active.parser_id.clone(),
        schema_id: active.schema_id.clone(),
        parser_fixture_schema_id: stage_metrics_contract(stage).metrics_schema_id.to_owned(),
        parser_fixture_parser_id: String::new(),
        parser_fixture_root_path: String::new(),
        expected_normalized_path: String::new(),
        raw_fixture_count: 0,
        raw_fixture_paths: Vec::new(),
        coverage_status: VcfParserFixtureCoverageStatus::MissingFixtureInventory,
        reason: format!(
            "active row `{}` / `{}` is missing a governed VCF parser fixture inventory binding",
            active.stage_id, active.tool_id
        ),
    };
    if let Some(fixture) = find_vcf_parser_fixture_inventory_row(&active.tool_id, stage) {
        let observation = super::fixtures::observe_fixture(repo_root, active, fixture)?;
        fixture.parser_id.clone_into(&mut row.parser_fixture_parser_id);
        fixture.fixture_path.clone_into(&mut row.parser_fixture_root_path);
        row.expected_normalized_path = format!("{}/expected.normalized.json", fixture.fixture_path);
        row.raw_fixture_count = observation.raw_fixture_paths.len();
        row.raw_fixture_paths = observation.raw_fixture_paths;
        row.coverage_status = observation.status;
        row.reason = observation.reason;
    }
    Ok(row)
}
