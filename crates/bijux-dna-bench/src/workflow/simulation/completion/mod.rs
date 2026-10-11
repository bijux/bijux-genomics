//! Owner: illustrative completion fixture orchestration over admitted sources and bounded IO.
use crate::repo::simulation::completion;
use anyhow::Result;
use bijux_dna_bench_model::{
    admit_all_domain_simulation_inputs,
    completion::{
        admit_all_domain_completion_check, AllDomainCompletionCheckReport,
        AllDomainCompletionStatus, ALL_DOMAIN_COMPLETION_CHECK_SCHEMA_VERSION,
        DEFAULT_ALL_DOMAIN_COMPLETION_CHECK_FIXTURE_ROOT,
    },
    AllDomainExpectedBenchmarkResultRow, AllDomainOutputDeclarationRow,
    AllDomainRenderedCommandRow,
};
use std::{collections::BTreeMap, path::Path};
mod mutation;
mod observation;

/// Materialize and inspect the five governed illustrative completion failures.
/// Commands are recorded without execution; unrelated fixture evidence survives reruns.
/// # Errors
/// Rejects contradictory sources and unsafe destinations before effects, or failed IO/admission.
pub fn render_all_domain_completion_check(
    repo_root: &Path,
    output_path: &Path,
    expected: &[AllDomainExpectedBenchmarkResultRow],
    commands: &[AllDomainRenderedCommandRow],
    outputs: &[AllDomainOutputDeclarationRow],
) -> Result<AllDomainCompletionCheckReport> {
    let inputs = admit_all_domain_simulation_inputs(expected, commands, outputs)?;
    let mutations =
        mutation::plan(&inputs, Path::new(DEFAULT_ALL_DOMAIN_COMPLETION_CHECK_FIXTURE_ROOT))?;
    let paths = completion::prepare(repo_root, output_path, &inputs)?;
    let simulated = super::fake_run_all_domain_benchmark_results(
        &paths.repository,
        &paths.root,
        expected,
        commands,
        outputs,
    )?;
    let seeded_mutations = mutation::apply(&paths, mutations)?;
    let rows = observation::collect(&paths, &inputs, &simulated.results)?;
    let mut domain_counts = BTreeMap::new();
    let mut failure_reason_counts = BTreeMap::new();
    let mut complete_row_count = 0;
    for row in &rows {
        *domain_counts.entry(row.domain.clone()).or_insert(0_usize) += 1;
        for reason in &row.failure_reasons {
            *failure_reason_counts.entry(reason.clone()).or_insert(0_usize) += 1;
        }
        complete_row_count +=
            usize::from(row.completion_status == AllDomainCompletionStatus::Complete);
    }
    let report = admit_all_domain_completion_check(AllDomainCompletionCheckReport {
        schema_version: ALL_DOMAIN_COMPLETION_CHECK_SCHEMA_VERSION.to_owned(),
        output_path: paths.display(&paths.report)?,
        fixture_root: paths.display(&paths.root)?,
        row_count: rows.len(),
        complete_row_count,
        incomplete_row_count: rows.len() - complete_row_count,
        domain_counts,
        failure_reason_counts,
        passes_behavior_test: false,
        seeded_mutations,
        rows,
    })?;
    completion::publish(&paths, &report)?;
    Ok(report)
}

#[cfg(test)]
#[path = "../../../../tests/contracts/simulation/completion.rs"]
mod tests;
