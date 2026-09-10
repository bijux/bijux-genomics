//! Owner: all-domain benchmark simulation admission and materialization workflow.
use crate::repo::simulation::{
    admit_destination, fake_run_result, path_relative_to_repo, publish_simulation_manifest,
    result_root,
};
use anyhow::Result;
use bijux_dna_bench_model::{
    admit_all_domain_simulation_inputs, all_domain_simulation_output_ids,
    all_domain_simulation_output_path, validate_all_domain_simulation_report,
    AllDomainExpectedBenchmarkResultRow, AllDomainFakeRunsReport, AllDomainOutputDeclarationRow,
    AllDomainRenderedCommandRow, ALL_DOMAIN_FAKE_RUNS_SCHEMA_VERSION,
};
use std::{collections::BTreeMap, path::Path};

/// Materialize illustrative benchmark outputs from admitted canonical rows; recorded commands are not executed.
///
/// # Errors
/// Rejects contradictory inputs, unsafe destinations and failed artifact publication.
pub fn fake_run_all_domain_benchmark_results(
    repo_root: &Path,
    output_root: &Path,
    expected: &[AllDomainExpectedBenchmarkResultRow],
    commands: &[AllDomainRenderedCommandRow],
    outputs: &[AllDomainOutputDeclarationRow],
) -> Result<AllDomainFakeRunsReport> {
    let absolute_output_root = admit_destination(repo_root, output_root)?;
    let inputs = admit_all_domain_simulation_inputs(expected, commands, outputs)?;
    // Admit the entire selected set before the first filesystem mutation.
    for input in &inputs {
        let expected = input.expected;
        let output = input.outputs;
        let destination = result_root(&absolute_output_root, expected);
        for path in ["command.sh", "stdout.txt", "stderr.txt", "metrics.json", "stage-result.json"]
        {
            admit_destination(repo_root, &destination.join(path))?;
        }
        for id in all_domain_simulation_output_ids(output) {
            let path =
                destination.join("declared-outputs").join(all_domain_simulation_output_path(&id));
            admit_destination(repo_root, &path)?;
            if id.ends_with("_bundle") || id.ends_with("_dir") {
                admit_destination(repo_root, &path.join(".bijux-all-domain-fake-run-placeholder"))?;
            }
        }
    }
    admit_destination(repo_root, &absolute_output_root.join("manifest.json"))?;

    let mut results = Vec::with_capacity(inputs.len());
    let mut created_output_count = 0usize;
    let mut domain_counts = BTreeMap::<String, usize>::new();

    for input in inputs {
        let report = fake_run_result(
            repo_root,
            &absolute_output_root,
            input.expected,
            input.command,
            input.outputs,
        )?;
        created_output_count += report.created_output_count;
        *domain_counts.entry(report.domain.clone()).or_default() += 1;
        results.push(report);
    }

    results.sort_by(|left, right| {
        left.domain
            .cmp(&right.domain)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
            .then_with(|| left.result_id.cmp(&right.result_id))
    });

    let root_manifest_path = absolute_output_root.join("manifest.json");
    let manifest = AllDomainFakeRunsReport {
        schema_version: ALL_DOMAIN_FAKE_RUNS_SCHEMA_VERSION.to_string(),
        fake_run_root: path_relative_to_repo(repo_root, &absolute_output_root),
        root_manifest_path: path_relative_to_repo(repo_root, &root_manifest_path),
        result_count: results.len(),
        created_output_count,
        domain_counts,
        results,
    };
    validate_all_domain_simulation_report(&manifest)?;
    publish_simulation_manifest(&root_manifest_path, &manifest)?;
    Ok(manifest)
}

#[cfg(test)]
#[path = "../../../tests/contracts/simulation.rs"]
mod tests;

mod failure;
pub use failure::fake_run_all_domain_failure_results;
