use super::{envelope::validate_expected_normalized_fixture, traversal::collect_raw_fixture_paths};
use anyhow::Result;
use bijux_dna_bench_vcf_evidence::model::{
    parsers::VcfParserFixtureCoverageStatus, scope::VcfActiveStageToolMatrixRow,
};
use bijux_dna_domain_vcf::VcfParserFixtureInventoryRow;
use std::path::Path;
pub(super) struct FixtureObservation {
    pub(super) raw_fixture_paths: Vec<String>,
    pub(super) status: VcfParserFixtureCoverageStatus,
    pub(super) reason: String,
}
pub(super) fn observe_fixture(
    repo_root: &Path,
    active: &VcfActiveStageToolMatrixRow,
    fixture: &VcfParserFixtureInventoryRow,
) -> Result<FixtureObservation> {
    let root = repo_root.join(fixture.fixture_path);
    let mut raw_fixture_paths = Vec::new();
    let (status, reason) = if root.is_dir() {
        raw_fixture_paths = collect_raw_fixture_paths(repo_root, &root)?;
        if raw_fixture_paths.is_empty() {
            (
                VcfParserFixtureCoverageStatus::MissingRawFixtures,
                format!(
                    "active row `{}` / `{}` fixture directory `{}` does not contain governed `raw.*` files",
                    active.stage_id, active.tool_id, fixture.fixture_path
                ),
            )
        } else if !root.join("expected.normalized.json").is_file() {
            (
                VcfParserFixtureCoverageStatus::MissingExpectedNormalizedJson,
                format!(
                    "active row `{}` / `{}` fixture directory `{}` is missing `expected.normalized.json`",
                    active.stage_id, active.tool_id, fixture.fixture_path
                ),
            )
        } else {
            let expected = format!("{}/expected.normalized.json", fixture.fixture_path);
            match validate_expected_normalized_fixture(
                &root.join("expected.normalized.json"),
                &active.stage_id,
                &active.tool_id,
                fixture.parser_id,
            ) {
                Ok(()) => (
                    VcfParserFixtureCoverageStatus::Covered,
                    format!(
                        "active row `{}` / `{}` keeps {} governed raw fixture file(s) under `{}` plus expected normalized JSON `{}`",
                        active.stage_id, active.tool_id, raw_fixture_paths.len(), fixture.fixture_path, expected
                    ),
                ),
                Err(error) => (
                    VcfParserFixtureCoverageStatus::InvalidExpectedNormalizedJson,
                    format!(
                        "active row `{}` / `{}` has invalid expected normalized JSON `{}`: {}",
                        active.stage_id, active.tool_id, expected, error
                    ),
                ),
            }
        }
    } else {
        (
            VcfParserFixtureCoverageStatus::MissingFixtureDirectory,
            format!(
                "active row `{}` / `{}` expects parser fixture directory `{}` but it is missing",
                active.stage_id, active.tool_id, fixture.fixture_path
            ),
        )
    };
    Ok(FixtureObservation { raw_fixture_paths, status, reason })
}
#[must_use]
pub fn coverage_status_label(status: VcfParserFixtureCoverageStatus) -> &'static str {
    match status {
        VcfParserFixtureCoverageStatus::Covered => "covered",
        VcfParserFixtureCoverageStatus::MissingFixtureInventory => "missing_fixture_inventory",
        VcfParserFixtureCoverageStatus::MissingFixtureDirectory => "missing_fixture_directory",
        VcfParserFixtureCoverageStatus::MissingExpectedNormalizedJson => {
            "missing_expected_normalized_json"
        }
        VcfParserFixtureCoverageStatus::MissingRawFixtures => "missing_raw_fixtures",
        VcfParserFixtureCoverageStatus::InvalidExpectedNormalizedJson => {
            "invalid_expected_normalized_json"
        }
    }
}
