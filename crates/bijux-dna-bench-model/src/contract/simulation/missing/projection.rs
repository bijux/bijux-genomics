//! Owner: projection of typed manifest observations into illustrative missing-result rows.
use crate::{
    AllDomainExpectedBenchmarkResultRow, AllDomainFakeRunResultReport, AllDomainMissingResultRow,
    AllDomainMissingResultStatus, AllDomainOutputDeclarationRow, AllDomainOutputDeclarationStatus,
    BenchmarkStageResultManifest, BenchmarkStageResultStatus,
};
use anyhow::{ensure, Result};
use std::collections::BTreeSet;

/// Bind a manifest observation to the exact declared and simulated source identities.
/// Absence describes the manifest observation; it does not assert scientific execution.
#[derive(Clone, Copy)]
pub struct AllDomainMissingResultObservation<'a> {
    pub expected: &'a AllDomainExpectedBenchmarkResultRow,
    pub outputs: &'a AllDomainOutputDeclarationRow,
    pub simulated: &'a AllDomainFakeRunResultReport,
    pub manifest: Option<&'a BenchmarkStageResultManifest>,
}

/// Project one admitted source observation without filesystem access or command execution.
/// # Errors
/// Rejects misbound identities, incomplete declarations and contradictory manifest contents.
pub fn project_all_domain_missing_result_observation(
    input: AllDomainMissingResultObservation<'_>,
) -> Result<AllDomainMissingResultRow> {
    let expected = input.expected;
    let output = input.outputs;
    let simulated = input.simulated;
    crate::validate_all_domain_simulation_result(simulated)?;
    ensure!(
        output.status == AllDomainOutputDeclarationStatus::Complete,
        "missing-result projection requires complete output declarations"
    );
    for (left, middle, right) in [
        (&expected.result_id, &output.result_id, &simulated.result_id),
        (&expected.domain, &output.domain, &simulated.domain),
        (&expected.stage_id, &output.stage_id, &simulated.stage_id),
        (&expected.tool_id, &output.tool_id, &simulated.tool_id),
        (&expected.corpus_id, &output.corpus_id, &simulated.corpus_id),
        (&expected.asset_profile_id, &output.asset_profile_id, &simulated.asset_profile_id),
    ] {
        ensure!(
            left == middle && left == right,
            "missing-result observation identities differ across source rows"
        );
    }
    let expected_output_artifact_ids = crate::all_domain_simulation_output_ids(output);
    ensure!(
        !expected.report_section.trim().is_empty()
            && !expected.expected_outputs.is_empty()
            && !expected.expected_metrics.is_empty()
            && expected.expected_metrics.iter().all(|metric| !metric.trim().is_empty()),
        "missing-result source has incomplete expected metrics or report metadata"
    );
    ensure!(
        simulated.expected_metric_count == expected.expected_metrics.len()
            && simulated.outputs.iter().map(|item| &item.artifact_id).collect::<BTreeSet<_>>()
                == expected_output_artifact_ids.iter().collect::<BTreeSet<_>>(),
        "missing-result simulation differs from declared output or metric coverage"
    );
    let (result_status, observed_output_artifact_ids, reason) = match input.manifest {
        None => (
            AllDomainMissingResultStatus::MissingResult,
            Vec::new(),
            format!(
                "expected all-domain benchmark result `{}` remains visible even though its fake-run manifest is missing",
                expected.result_id
            ),
        ),
        Some(manifest) => (
            AllDomainMissingResultStatus::Present,
            admit_manifest_outputs(expected, &expected_output_artifact_ids, manifest)?,
            format!(
                "expected all-domain benchmark result `{}` remains present with a validated fake-run manifest",
                expected.result_id
            ),
        ),
    };
    Ok(AllDomainMissingResultRow {
        result_id: expected.result_id.clone(),
        domain: expected.domain.clone(),
        stage_id: expected.stage_id.clone(),
        tool_id: expected.tool_id.clone(),
        corpus_id: expected.corpus_id.clone(),
        asset_profile_id: expected.asset_profile_id.clone(),
        expected_manifest_path: simulated.stage_result_path.clone(),
        audit_manifest_path: simulated.stage_result_path.clone(),
        result_status,
        expected_output_artifact_ids,
        observed_output_artifact_ids,
        expected_metrics: expected.expected_metrics.clone(),
        report_section: expected.report_section.clone(),
        reason,
    })
}

fn admit_manifest_outputs(
    expected: &AllDomainExpectedBenchmarkResultRow,
    expected_output_artifact_ids: &[String],
    manifest: &BenchmarkStageResultManifest,
) -> Result<Vec<String>> {
    crate::validate_benchmark_stage_result_manifest(manifest)?;
    ensure!(
        manifest.stage_id == expected.stage_id && manifest.tool.id == expected.tool_id,
        "missing-result manifest differs from its expected stage and tool"
    );
    ensure!(
        manifest.runtime.mode == "benchmark_fake_run"
            && manifest.runtime.status == BenchmarkStageResultStatus::Succeeded
            && manifest.runtime.exit_code == 0,
        "missing-result audit requires a successful illustrative manifest"
    );
    let observed =
        manifest.outputs.iter().map(|artifact| artifact.artifact_id.clone()).collect::<Vec<_>>();
    let observed_set = observed.iter().collect::<BTreeSet<_>>();
    let expected_set = expected_output_artifact_ids.iter().collect::<BTreeSet<_>>();
    ensure!(
        observed.len() == observed_set.len() && observed_set == expected_set,
        "missing-result manifest output identities differ from its declaration"
    );
    Ok(observed)
}
