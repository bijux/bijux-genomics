//! Owner: admitted illustrative omissions, current manifest observations and audit publication.
use crate::repo::simulation::missing::{self, MissingResultPaths};
use anyhow::{Context, Result};
use bijux_dna_bench_model::{
    admit_all_domain_missing_result_audit, admit_all_domain_missing_result_inputs,
    project_all_domain_missing_result_observation, AllDomainExpectedBenchmarkResultRow,
    AllDomainFakeRunResultReport, AllDomainMissingResultObservation, AllDomainMissingResultRow,
    AllDomainMissingResultStatus, AllDomainMissingResultTestReport, AllDomainOutputDeclarationRow,
    AllDomainRenderedCommandRow, ALL_DOMAIN_MISSING_RESULT_TEST_SCHEMA_VERSION,
    BAM_REMOVED_RESULT_ID, FASTQ_REMOVED_RESULT_ID, VCF_REMOVED_RESULT_ID,
};
use std::{collections::BTreeMap, path::Path};

/// Materialize an illustrative audit with three governed omissions and preserve unrelated files.
/// Source identities and every destination are admitted before filesystem mutation begins.
/// # Errors
/// Rejects malformed sources, unsafe paths and unreadable, nonregular or invalid manifests.
pub fn render_all_domain_missing_result_audit(
    repo_root: &Path,
    output_path: &Path,
    expected: &[AllDomainExpectedBenchmarkResultRow],
    commands: &[AllDomainRenderedCommandRow],
    outputs: &[AllDomainOutputDeclarationRow],
) -> Result<AllDomainMissingResultTestReport> {
    let inputs = admit_all_domain_missing_result_inputs(expected, commands, outputs)?;
    let paths = missing::prepare(repo_root, output_path, &inputs)?;
    let fake_runs = super::fake_run_all_domain_benchmark_results(
        &paths.repository,
        &paths.root,
        expected,
        commands,
        outputs,
    )?;
    let by_id = fake_runs
        .results
        .iter()
        .map(|row| (row.result_id.as_str(), row))
        .collect::<BTreeMap<_, _>>();
    let mut removed_result_ids = Vec::new();
    let mut removed_manifest_paths = Vec::new();
    for id in [FASTQ_REMOVED_RESULT_ID, BAM_REMOVED_RESULT_ID, VCF_REMOVED_RESULT_ID] {
        let row = by_id.get(id).context("admitted omission is absent from simulation")?;
        missing::remove_manifest(&paths, &row.stage_result_path)?;
        removed_result_ids.push(id.to_string());
        removed_manifest_paths.push(row.stage_result_path.clone());
    }
    let rows = observe_rows(&paths, expected, outputs, &by_id)?;
    let mut domain_counts = BTreeMap::new();
    for row in &rows {
        *domain_counts.entry(row.domain.clone()).or_insert(0_usize) += 1;
    }
    let missing_result_row_count = rows
        .iter()
        .filter(|row| row.result_status == AllDomainMissingResultStatus::MissingResult)
        .count();
    let report = admit_all_domain_missing_result_audit(AllDomainMissingResultTestReport {
        schema_version: ALL_DOMAIN_MISSING_RESULT_TEST_SCHEMA_VERSION.to_string(),
        output_path: paths.display(&paths.report)?,
        fake_result_root: paths.display(&paths.root)?,
        expected_row_count: rows.len(),
        present_result_row_count: rows.len() - missing_result_row_count,
        missing_result_row_count,
        passes_behavior_test: false,
        removed_result_ids,
        removed_manifest_paths,
        domain_counts,
        rows,
    })?;
    missing::publish(&paths, &report)?;
    Ok(report)
}

fn observe_rows(
    paths: &MissingResultPaths,
    expected: &[AllDomainExpectedBenchmarkResultRow],
    outputs: &[AllDomainOutputDeclarationRow],
    simulated: &BTreeMap<&str, &AllDomainFakeRunResultReport>,
) -> Result<Vec<AllDomainMissingResultRow>> {
    let outputs =
        outputs.iter().map(|row| (row.result_id.as_str(), row)).collect::<BTreeMap<_, _>>();
    expected
        .iter()
        .map(|row| {
            let source = simulated
                .get(row.result_id.as_str())
                .context("admitted simulation result is absent")?;
            let output = outputs
                .get(row.result_id.as_str())
                .context("admitted output declaration is absent")?;
            let manifest = missing::read_manifest(paths, &source.stage_result_path)?;
            project_all_domain_missing_result_observation(AllDomainMissingResultObservation {
                expected: row,
                outputs: output,
                simulated: source,
                manifest: manifest.as_ref(),
            })
        })
        .collect()
}
