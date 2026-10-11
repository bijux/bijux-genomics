//! Owner: Failure simulation admission and publication; recorded tool commands are never executed.
use crate::repo::simulation::failure::{
    prepare_failure_record, publish_failure_manifest, publish_failure_record,
};
use crate::repo::simulation::{admit_destination, path_relative_to_repo};
use anyhow::{ensure, Context, Result};
use bijux_dna_bench_model::{
    admit_all_domain_simulation_inputs, validate_all_domain_failure_simulation_manifest,
    AllDomainExpectedBenchmarkResultRow, AllDomainFakeFailuresManifest,
    AllDomainOutputDeclarationRow, AllDomainRenderedCommandRow,
    ALL_DOMAIN_FAKE_FAILURE_MANIFEST_SCHEMA_VERSION,
};
use std::collections::BTreeMap;
use std::path::Path;

/// Materialize illustrative failure artifacts from complete, uniquely aligned source rows.
/// Every declared input and destination is admitted before publication begins.
/// # Errors
/// Rejects zero exit codes, malformed/duplicate/misbound inputs, unsafe paths and failed IO.
pub fn fake_run_all_domain_failure_results(
    repo_root: &Path,
    output_root: &Path,
    expected: &[AllDomainExpectedBenchmarkResultRow],
    commands: &[AllDomainRenderedCommandRow],
    outputs: &[AllDomainOutputDeclarationRow],
    exit_code: i32,
) -> Result<AllDomainFakeFailuresManifest> {
    ensure!(exit_code != 0, "failure simulation requires a nonzero exit code");
    let root = admit_destination(repo_root, output_root)?;
    ensure!(
        root.strip_prefix(repo_root)?.to_str().is_some(),
        "failure simulation destination must have a UTF-8 repository-relative path"
    );
    let inputs = admit_all_domain_simulation_inputs(expected, commands, outputs)?;
    let mut records = Vec::with_capacity(inputs.len());
    let mut domains = BTreeMap::new();
    let mut output_count = 0_usize;
    for input in &inputs {
        let record = prepare_failure_record(repo_root, &root, input, exit_code)?;
        for path in [&record.command_script_path, &record.stderr_path, &record.failure_record_path]
        {
            admit_destination(repo_root, Path::new(path))?;
        }
        for output in &record.failed_outputs {
            admit_destination(repo_root, Path::new(&output.expected_fake_run_path))?;
        }
        output_count = output_count
            .checked_add(record.failed_output_count)
            .context("failure output total overflow")?;
        *domains.entry(record.domain.clone()).or_insert(0_usize) += 1;
        records.push(record);
    }
    let manifest_path = admit_destination(repo_root, &root.join("manifest.json"))?;
    // Retain the historic canonical record ordering independently of acquisition order.
    let mut failures = records;
    failures.sort_by(|left, right| {
        left.domain
            .cmp(&right.domain)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
            .then_with(|| left.result_id.cmp(&right.result_id))
    });
    let manifest = AllDomainFakeFailuresManifest {
        schema_version: ALL_DOMAIN_FAKE_FAILURE_MANIFEST_SCHEMA_VERSION.to_string(),
        failure_root: path_relative_to_repo(repo_root, &root),
        root_manifest_path: path_relative_to_repo(repo_root, &manifest_path),
        result_count: failures.len(),
        failed_output_count: output_count,
        exit_code,
        domain_counts: domains,
        failures,
    };
    validate_all_domain_failure_simulation_manifest(&manifest)?;
    let by_identity = manifest
        .failures
        .iter()
        .map(|record| (record.result_id.as_str(), record))
        .collect::<BTreeMap<_, _>>();
    for input in &inputs {
        let record = by_identity
            .get(input.expected.result_id.as_str())
            .context("admitted failure record is absent")?;
        publish_failure_record(repo_root, input, record)?;
    }
    publish_failure_manifest(&manifest_path, &manifest)?;
    Ok(manifest)
}
