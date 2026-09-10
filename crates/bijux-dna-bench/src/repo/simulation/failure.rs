//! Owner: Filesystem path projection and publication for illustrative failure records.
use super::{admit_destination, path_relative_to_repo, result_root};
use crate::artifacts::simulation::render_all_domain_simulation_command_script;
use anyhow::{Context, Result};
use bijux_dna_bench_model::{
    all_domain_simulation_output_ids, all_domain_simulation_output_path,
    all_domain_simulation_output_role, validate_all_domain_failure_simulation_record,
    AllDomainFakeFailureOutputEntry, AllDomainFakeFailureRecord, AllDomainFakeFailuresManifest,
    AllDomainSimulationInput, ALL_DOMAIN_FAKE_FAILURE_RECORD_SCHEMA_VERSION,
};
use std::path::Path;

pub(crate) fn prepare_failure_record(
    repo_root: &Path,
    failure_root: &Path,
    input: &AllDomainSimulationInput<'_>,
    exit_code: i32,
) -> Result<AllDomainFakeFailureRecord> {
    let expected = input.expected;
    let command = input.command;
    let destination = result_root(failure_root, expected);
    let failed_outputs = all_domain_simulation_output_ids(input.outputs)
        .into_iter()
        .map(|artifact_id| AllDomainFakeFailureOutputEntry {
            expected_fake_run_path: path_relative_to_repo(
                repo_root,
                &destination
                    .join("declared-outputs")
                    .join(all_domain_simulation_output_path(&artifact_id)),
            ),
            role: all_domain_simulation_output_role(&artifact_id).to_string(),
            declared_output: artifact_id.clone(),
            artifact_id,
        })
        .collect::<Vec<_>>();
    let record = AllDomainFakeFailureRecord {
        schema_version: ALL_DOMAIN_FAKE_FAILURE_RECORD_SCHEMA_VERSION.to_string(),
        result_id: expected.result_id.clone(),
        domain: expected.domain.clone(),
        stage_id: expected.stage_id.clone(),
        tool_id: expected.tool_id.clone(),
        corpus_id: expected.corpus_id.clone(),
        asset_profile_id: expected.asset_profile_id.clone(),
        command_source: command.command_source.clone(),
        command: command.script_commands.join("\n"),
        exit_code,
        command_script_path: path_relative_to_repo(repo_root, &destination.join("command.sh")),
        stderr_path: path_relative_to_repo(repo_root, &destination.join("stderr.txt")),
        failure_record_path: path_relative_to_repo(repo_root, &destination.join("failure.json")),
        failed_output_count: failed_outputs.len(),
        failed_outputs,
    };
    validate_all_domain_failure_simulation_record(&record)?;
    Ok(record)
}

pub(crate) fn publish_failure_record(
    repo_root: &Path,
    input: &AllDomainSimulationInput<'_>,
    record: &AllDomainFakeFailureRecord,
) -> Result<()> {
    let script = admit_destination(repo_root, Path::new(&record.command_script_path))?;
    let stderr = admit_destination(repo_root, Path::new(&record.stderr_path))?;
    let output = admit_destination(repo_root, Path::new(&record.failure_record_path))?;
    bijux_dna_infra::ensure_dir(output.parent().context("failure record has no parent")?)?;
    bijux_dna_infra::write_bytes(
        &script,
        render_all_domain_simulation_command_script(input.expected, input.command).as_bytes(),
    )
    .with_context(|| format!("write {}", script.display()))?;
    let diagnostic=format!("fake local all-domain benchmark failure\nresult_id={}\ndomain={}\nstage_id={}\ntool_id={}\nexit_code={}\ncommand={}\n",
        record.result_id,record.domain,record.stage_id,record.tool_id,record.exit_code,input.command.script_commands.join(" && "));
    bijux_dna_infra::write_bytes(&stderr, diagnostic.as_bytes())
        .with_context(|| format!("write {}", stderr.display()))?;
    bijux_dna_infra::atomic_write_json(&output, record)?;
    Ok(())
}
pub(crate) fn publish_failure_manifest(
    path: &Path,
    manifest: &AllDomainFakeFailuresManifest,
) -> Result<()> {
    bijux_dna_infra::atomic_write_json(path, manifest)
        .context("publish all-domain failure simulation manifest")
}
