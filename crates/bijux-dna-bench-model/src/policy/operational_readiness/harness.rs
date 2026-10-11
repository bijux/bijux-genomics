//! Reconcile explicit local harness report identities without acquiring or publishing artifacts.
use crate::{
    completion::AllDomainCompletionCheckReport, AllDomainExpectedBenchmarkResultsReport,
    AllDomainFakeFailuresManifest, AllDomainFakeRunsReport, AllDomainMissingResultTestReport,
    AllDomainOutputDeclarationsReport, AllDomainParserCollectorReport,
    AllDomainParserCollectorSourceKind, AllDomainRenderedCommandsReport,
    AllDomainStageToolTableReport,
};
use anyhow::{bail, Result};
use std::collections::BTreeSet;

/// Borrow current reports for exact result and benchmark-ready binding coverage.
#[derive(Clone, Copy)]
pub struct HarnessAlignmentEvidence<'a> {
    pub stage_tool_report: &'a AllDomainStageToolTableReport,
    pub expected_results_report: &'a AllDomainExpectedBenchmarkResultsReport,
    pub rendered_commands_report: &'a AllDomainRenderedCommandsReport,
    pub output_declarations_report: &'a AllDomainOutputDeclarationsReport,
    pub fake_runs_report: &'a AllDomainFakeRunsReport,
    pub fake_failures_report: &'a AllDomainFakeFailuresManifest,
    pub completion_check_report: &'a AllDomainCompletionCheckReport,
    pub parser_collector_report: &'a AllDomainParserCollectorReport,
    pub missing_result_report: &'a AllDomainMissingResultTestReport,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct HarnessBindingKey<'a> {
    domain: &'a str,
    stage_id: &'a str,
    tool_id: &'a str,
    corpus_id: &'a str,
    asset_profile_id: &'a str,
}

/// Require the same unique result identities across all simulation report families.
/// Real-smoke parser records have a separate identity space and are excluded.
///
/// # Errors
/// Rejects blank or repeated result IDs, missing/extra result coverage, and binding drift.
pub fn validate_all_domain_harness_alignment(evidence: HarnessAlignmentEvidence<'_>) -> Result<()> {
    let HarnessAlignmentEvidence {
        stage_tool_report,
        expected_results_report,
        rendered_commands_report,
        output_declarations_report,
        fake_runs_report,
        fake_failures_report,
        completion_check_report,
        parser_collector_report,
        missing_result_report,
    } = evidence;
    let expected_result_ids = unique_result_ids(
        "expected results",
        expected_results_report.rows.iter().map(|row| row.result_id.as_str()),
    )?;
    let rendered_result_ids = unique_result_ids(
        "rendered commands",
        rendered_commands_report.rows.iter().map(|row| row.result_id.as_str()),
    )?;
    let output_result_ids = unique_result_ids(
        "output declarations",
        output_declarations_report.rows.iter().map(|row| row.result_id.as_str()),
    )?;
    let fake_run_result_ids = unique_result_ids(
        "fake runs",
        fake_runs_report.results.iter().map(|row| row.result_id.as_str()),
    )?;
    let fake_failure_result_ids = unique_result_ids(
        "fake failures",
        fake_failures_report.failures.iter().map(|row| row.result_id.as_str()),
    )?;
    let completion_result_ids = unique_result_ids(
        "completion rows",
        completion_check_report.rows.iter().map(|row| row.result_id.as_str()),
    )?;
    let missing_result_ids = unique_result_ids(
        "missing-result rows",
        missing_result_report.rows.iter().map(|row| row.result_id.as_str()),
    )?;

    ensure_matching_set("rendered commands", &expected_result_ids, &rendered_result_ids)?;
    ensure_matching_set("output declarations", &expected_result_ids, &output_result_ids)?;
    ensure_matching_set("fake runs", &expected_result_ids, &fake_run_result_ids)?;
    ensure_matching_set("fake failures", &expected_result_ids, &fake_failure_result_ids)?;
    ensure_matching_set("completion rows", &expected_result_ids, &completion_result_ids)?;
    ensure_matching_set("missing-result rows", &expected_result_ids, &missing_result_ids)?;

    let parser_fake_run_result_ids = unique_result_ids(
        "parser collector fake-run rows",
        parser_collector_report
            .rows
            .iter()
            .filter(|row| row.source_kind == AllDomainParserCollectorSourceKind::FakeRun)
            .map(|row| row.result_id.as_deref().unwrap_or_default()),
    )?;
    ensure_matching_set(
        "parser collector fake-run rows",
        &expected_result_ids,
        &parser_fake_run_result_ids,
    )?;

    let expected_bindings =
        expected_results_report.rows.iter().map(expected_binding_key).collect::<BTreeSet<_>>();
    let stage_tool_bindings = stage_tool_report
        .rows
        .iter()
        .filter(|row| row.benchmark_status == "benchmark_ready")
        .map(stage_tool_binding_key)
        .collect::<BTreeSet<_>>();
    ensure_matching_binding_set("stage tool table", &expected_bindings, &stage_tool_bindings)?;

    Ok(())
}
fn unique_result_ids<'a>(
    label: &str,
    ids: impl IntoIterator<Item = &'a str>,
) -> Result<BTreeSet<&'a str>> {
    let mut unique = BTreeSet::new();
    for id in ids {
        if id.trim().is_empty() || id.trim() != id {
            bail!("{label} has a blank or padded result identity");
        }
        if !unique.insert(id) {
            bail!("{label} repeats result identity {id}");
        }
    }
    Ok(unique)
}

fn ensure_matching_set(
    label: &str,
    expected: &BTreeSet<&str>,
    observed: &BTreeSet<&str>,
) -> Result<()> {
    if expected != observed {
        let missing = expected.difference(observed).copied().collect::<Vec<_>>();
        let extra = observed.difference(expected).copied().collect::<Vec<_>>();
        bail!("{label} drifted from the canonical all-domain result slice; missing={missing:?} extra={extra:?}");
    }
    Ok(())
}

fn ensure_matching_binding_set(
    label: &str,
    expected: &BTreeSet<HarnessBindingKey<'_>>,
    observed: &BTreeSet<HarnessBindingKey<'_>>,
) -> Result<()> {
    if expected != observed {
        let missing = expected.difference(observed).cloned().collect::<Vec<_>>();
        let extra = observed.difference(expected).cloned().collect::<Vec<_>>();
        bail!("{label} drifted from the canonical all-domain binding slice; missing={missing:?} extra={extra:?}");
    }
    Ok(())
}

fn expected_binding_key(row: &crate::AllDomainExpectedBenchmarkResultRow) -> HarnessBindingKey<'_> {
    HarnessBindingKey {
        domain: &row.domain,
        stage_id: &row.stage_id,
        tool_id: &row.tool_id,
        corpus_id: &row.corpus_id,
        asset_profile_id: &row.asset_profile_id,
    }
}

fn stage_tool_binding_key(row: &crate::AllDomainStageToolTableRow) -> HarnessBindingKey<'_> {
    HarnessBindingKey {
        domain: &row.domain,
        stage_id: &row.stage_id,
        tool_id: &row.tool_id,
        corpus_id: &row.corpus_id,
        asset_profile_id: &row.asset_profile_id,
    }
}

#[cfg(test)]
#[path = "../../../tests/contracts/harness_alignment.rs"]
mod tests;
