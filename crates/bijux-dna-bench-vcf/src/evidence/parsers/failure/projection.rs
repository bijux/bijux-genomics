use super::cases::{VcfParserFailureCase, VcfParserFailureReason, VCF_PARSER_FAILURE_CASES};
use super::fixtures::{admit_fixture_set, materialize_fixture, publish_probe};
use super::model::VcfParserFailureTestRow;
use super::mutation::materialize_failure_probe;
use super::parsing::parse_case;
use anyhow::Context;
use anyhow::Result;
use std::path::Path;
pub(super) fn collect_vcf_parser_failure_rows(
    repo_root: &Path,
) -> Result<Vec<VcfParserFailureTestRow>> {
    let fixtures = admit_fixture_set(repo_root, VCF_PARSER_FAILURE_CASES)?;
    let published_root = repo_root.join("artifacts/bench-readiness/vcf-parser-failure-tests");
    let working_root_parent = published_root.join("runs");
    bijux_dna_infra::ensure_dir(&working_root_parent)
        .with_context(|| format!("create {}", working_root_parent.display()))?;
    let working_root = bijux_dna_infra::temp_dir_in(&working_root_parent, "run-")
        .with_context(|| format!("create scratch root under {}", working_root_parent.display()))?;

    let mut rows = VCF_PARSER_FAILURE_CASES
        .iter()
        .zip(&fixtures)
        .map(|(case, fixture)| {
            evaluate_case(repo_root, working_root.path(), &published_root, case, fixture)
        })
        .collect::<Result<Vec<_>>>()?;
    rows.sort_by(|left, right| {
        left.stage_id
            .cmp(&right.stage_id)
            .then_with(|| left.tool_id.cmp(&right.tool_id))
            .then_with(|| left.parser_id.cmp(&right.parser_id))
            .then_with(|| left.failure_reason.cmp(&right.failure_reason))
    });
    Ok(rows)
}
fn evaluate_case(
    repo_root: &Path,
    working_root: &Path,
    published_root: &Path,
    case: &VcfParserFailureCase,
    fixture: &super::fixtures::FixtureSnapshot,
) -> Result<VcfParserFailureTestRow> {
    let fixture_source = repo_root.join(case.fixture_dir);
    let working_probe_root = working_root.join(case.case_id).join("fixture");
    materialize_fixture(fixture, &working_probe_root)?;
    materialize_failure_probe(&working_probe_root, case)?;
    let published_probe_root = published_root.join(case.case_id).join("fixture");
    publish_probe(repo_root, &working_probe_root, &published_probe_root, case)?;
    let published_probe_artifact_path = published_probe_root.join(case.probe_artifact);
    let observed = parse_case(case, &working_probe_root);
    let observed_error = match observed {
        Ok(()) => {
            format!("parser unexpectedly accepted {}", published_probe_artifact_path.display())
        }
        Err(error) => normalize_probe_error_path(
            &error.to_string(),
            &working_probe_root,
            &published_probe_root,
        ),
    };
    let passed = observed_error.contains(case.expected_error_fragment)
        && !observed_error.starts_with("parser unexpectedly accepted");

    Ok(VcfParserFailureTestRow {
        case_id: case.case_id.to_string(),
        stage_id: case.stage.as_str().to_string(),
        tool_id: case.tool_id.to_string(),
        parser_id: case.parser_id.to_string(),
        failure_reason: failure_reason_label(case.failure_reason).to_string(),
        fixture_path: path_relative_to_repo(repo_root, &fixture_source),
        probe_artifact_path: path_relative_to_repo(repo_root, &published_probe_artifact_path),
        expected_error_fragment: case.expected_error_fragment.to_string(),
        observed_error,
        passed,
    })
}
fn failure_reason_label(value: VcfParserFailureReason) -> &'static str {
    match value {
        VcfParserFailureReason::EmptyOutput => "empty_output",
        VcfParserFailureReason::MalformedVcf => "malformed_vcf",
        VcfParserFailureReason::MissingIndex => "missing_index",
        VcfParserFailureReason::MissingSampleColumn => "missing_sample_column",
        VcfParserFailureReason::MalformedPcaTable => "malformed_pca_table",
        VcfParserFailureReason::MalformedImputationQualityFile => {
            "malformed_imputation_quality_file"
        }
        VcfParserFailureReason::MalformedSegmentFile => "malformed_segment_file",
    }
}
fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}
fn normalize_probe_error_path(message: &str, working_root: &Path, published_root: &Path) -> String {
    let working_root = working_root.to_string_lossy();
    let published_root = published_root.to_string_lossy();
    message.replace(working_root.as_ref(), published_root.as_ref())
}
