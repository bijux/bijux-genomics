//! Owner: local benchmark simulation output storage and persisted artifact materialization.
use crate::artifacts::simulation::{
    fake_output_bytes, output_path_is_directory, render_all_domain_simulation_command_script,
    render_result_stdout,
};
use anyhow::{anyhow, Context, Result};
use bijux_dna_bench_model::{
    all_domain_simulation_elapsed_seconds, all_domain_simulation_output_ids,
    all_domain_simulation_output_path, all_domain_simulation_output_role,
    validate_all_domain_simulation_result,
    validate_benchmark_stage_result_manifest as validate_stage_result_manifest,
    AllDomainExpectedBenchmarkResultRow, AllDomainFakeRunMetrics, AllDomainFakeRunOutputEntry,
    AllDomainFakeRunResultReport, AllDomainFakeRunsReport, AllDomainOutputDeclarationRow,
    AllDomainOutputDeclarationStatus, AllDomainRenderedCommandRow,
    BenchmarkStageResultCommand as BenchStageResultCommandV1,
    BenchmarkStageResultManifest as BenchStageResultManifestV1,
    BenchmarkStageResultOutput as BenchStageResultOutputV1,
    BenchmarkStageResultResourceMetricSource as BenchStageResultResourceMetricSource,
    BenchmarkStageResultResourceMetrics as BenchStageResultResourceMetricsV1,
    BenchmarkStageResultRuntime as BenchStageResultRuntimeV1,
    BenchmarkStageResultStatus as BenchStageResultStatus,
    BenchmarkStageResultTool as BenchStageResultToolV1, ALL_DOMAIN_FAKE_RUN_METRICS_SCHEMA_VERSION,
    BENCHMARK_STAGE_RESULT_SCHEMA_V2 as BENCH_STAGE_RESULT_SCHEMA_VERSION,
};
use std::{
    fs,
    path::{Path, PathBuf},
};
struct ResultFakeRunArtifacts {
    command_script_path: PathBuf,
    stdout_path: PathBuf,
    stderr_path: PathBuf,
    metrics_path: PathBuf,
    stage_result_path: PathBuf,
}

pub(crate) fn fake_run_result(
    repo_root: &Path,
    fake_run_root: &Path,
    expected: &AllDomainExpectedBenchmarkResultRow,
    command: &AllDomainRenderedCommandRow,
    outputs: &AllDomainOutputDeclarationRow,
) -> Result<AllDomainFakeRunResultReport> {
    if outputs.status != AllDomainOutputDeclarationStatus::Complete {
        return Err(anyhow!(
            "all-domain fake-runner requires complete output declarations for `{}`",
            expected.result_id
        ));
    }

    let result_root = result_root(fake_run_root, expected);
    fs::create_dir_all(&result_root)
        .with_context(|| format!("create {}", result_root.display()))?;

    let artifacts = ResultFakeRunArtifacts {
        command_script_path: result_root.join("command.sh"),
        stdout_path: result_root.join("stdout.txt"),
        stderr_path: result_root.join("stderr.txt"),
        metrics_path: result_root.join("metrics.json"),
        stage_result_path: result_root.join("stage-result.json"),
    };

    fs::write(
        &artifacts.command_script_path,
        render_all_domain_simulation_command_script(expected, command),
    )
    .with_context(|| format!("write {}", artifacts.command_script_path.display()))?;
    fs::write(&artifacts.stdout_path, render_result_stdout(expected, command, outputs))
        .with_context(|| format!("write {}", artifacts.stdout_path.display()))?;
    fs::write(
        &artifacts.stderr_path,
        format!(
            "fake local all-domain benchmark run produced no stderr\nresult_id={}\ndomain={}\nstage_id={}\ntool_id={}\n",
            expected.result_id, expected.domain, expected.stage_id, expected.tool_id
        ),
    )
    .with_context(|| format!("write {}", artifacts.stderr_path.display()))?;

    let declared_outputs = all_domain_simulation_output_ids(outputs);
    let mut output_entries = Vec::with_capacity(declared_outputs.len());
    let mut materialized_byte_count = 0u64;
    for artifact_id in declared_outputs {
        let output_path = result_root
            .join("declared-outputs")
            .join(all_domain_simulation_output_path(&artifact_id));
        let byte_count = materialize_fake_run_output(&output_path, expected, &artifact_id)
            .with_context(|| format!("materialize `{artifact_id}` for `{}`", expected.result_id))?;
        let exists = output_path.exists();
        if exists {
            materialized_byte_count += byte_count;
        }
        output_entries.push(AllDomainFakeRunOutputEntry {
            artifact_id: artifact_id.clone(),
            declared_output: artifact_id.clone(),
            fake_run_path: path_relative_to_repo(repo_root, &output_path),
            role: all_domain_simulation_output_role(&artifact_id).to_string(),
            exists,
        });
    }

    let metrics = AllDomainFakeRunMetrics {
        schema_version: ALL_DOMAIN_FAKE_RUN_METRICS_SCHEMA_VERSION.to_string(),
        result_id: expected.result_id.clone(),
        domain: expected.domain.clone(),
        stage_id: expected.stage_id.clone(),
        tool_id: expected.tool_id.clone(),
        corpus_id: expected.corpus_id.clone(),
        asset_profile_id: expected.asset_profile_id.clone(),
        command_step_count: command.command_steps.len(),
        declared_output_count: output_entries.len(),
        expected_metric_count: expected.expected_metrics.len(),
        expected_metrics: expected.expected_metrics.clone(),
        materialized_byte_count,
        simulated_elapsed_seconds: all_domain_simulation_elapsed_seconds(
            command.command_steps.len(),
        ),
    };
    bijux_dna_infra::atomic_write_json(&artifacts.metrics_path, &metrics)?;

    let stage_result = BenchStageResultManifestV1 {
        schema_version: BENCH_STAGE_RESULT_SCHEMA_VERSION.to_string(),
        stage_id: expected.stage_id.clone(),
        tool: BenchStageResultToolV1 { id: expected.tool_id.clone() },
        command: BenchStageResultCommandV1 { rendered: command.script_commands.join("\n") },
        runtime: BenchStageResultRuntimeV1 {
            mode: "benchmark_fake_run".to_string(),
            status: BenchStageResultStatus::Succeeded,
            started_at: "1970-01-01T00:00:00Z".to_string(),
            finished_at: "1970-01-01T00:00:01Z".to_string(),
            elapsed_seconds: metrics.simulated_elapsed_seconds,
            exit_code: 0,
        },
        resource_metrics: BenchStageResultResourceMetricsV1 {
            source: BenchStageResultResourceMetricSource::NotAvailable,
            memory_mb: None,
            cpu_threads: None,
        },
        outputs: output_entries
            .iter()
            .map(|output| BenchStageResultOutputV1 {
                artifact_id: output.artifact_id.clone(),
                declared_path: output.declared_output.clone(),
                realized_path: output.fake_run_path.clone(),
                role: output.role.clone(),
                optional: false,
                exists: output.exists,
            })
            .collect(),
    };
    validate_stage_result_manifest(&stage_result)?;
    bijux_dna_infra::atomic_write_json(&artifacts.stage_result_path, &stage_result)?;

    let report = AllDomainFakeRunResultReport {
        result_id: expected.result_id.clone(),
        domain: expected.domain.clone(),
        stage_id: expected.stage_id.clone(),
        tool_id: expected.tool_id.clone(),
        corpus_id: expected.corpus_id.clone(),
        asset_profile_id: expected.asset_profile_id.clone(),
        command_source: command.command_source.clone(),
        command_step_count: command.command_steps.len(),
        declared_output_count: output_entries.len(),
        created_output_count: output_entries.iter().filter(|output| output.exists).count(),
        expected_metric_count: expected.expected_metrics.len(),
        command_script_path: path_relative_to_repo(repo_root, &artifacts.command_script_path),
        stdout_path: path_relative_to_repo(repo_root, &artifacts.stdout_path),
        stderr_path: path_relative_to_repo(repo_root, &artifacts.stderr_path),
        metrics_path: path_relative_to_repo(repo_root, &artifacts.metrics_path),
        stage_result_path: path_relative_to_repo(repo_root, &artifacts.stage_result_path),
        outputs: output_entries,
    };
    validate_all_domain_simulation_result(&report)?;
    Ok(report)
}
pub(crate) fn result_root(root: &Path, row: &AllDomainExpectedBenchmarkResultRow) -> PathBuf {
    root.join(&row.domain)
        .join(&row.corpus_id)
        .join(&row.stage_id)
        .join(&row.asset_profile_id)
        .join(&row.tool_id)
}
pub(crate) fn materialize_fake_run_output(
    output_path: &Path,
    expected: &AllDomainExpectedBenchmarkResultRow,
    artifact_id: &str,
) -> Result<u64> {
    if output_path_is_directory(artifact_id) {
        fs::create_dir_all(output_path)
            .with_context(|| format!("create {}", output_path.display()))?;
        let sentinel = output_path.join(".bijux-all-domain-fake-run-placeholder");
        fs::write(
            &sentinel,
            format!(
                "all-domain fake-run directory placeholder\nresult_id={}\nstage_id={}\nartifact_id={artifact_id}\n",
                expected.result_id, expected.stage_id
            ),
        )
        .with_context(|| format!("write {}", sentinel.display()))?;
        return Ok(fs::metadata(&sentinel)?.len());
    }

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(output_path, fake_output_bytes(expected, artifact_id, output_path)?)
        .with_context(|| format!("write {}", output_path.display()))?;
    Ok(fs::metadata(output_path)?.len())
}

pub(crate) fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).display().to_string()
}

/// Admit a simulation destination before any write; existing symlinks are not output owners.
pub(crate) fn admit_destination(repo_root: &Path, output_root: &Path) -> Result<PathBuf> {
    let absolute = if output_root.is_absolute() {
        output_root.to_path_buf()
    } else {
        repo_root.join(output_root)
    };
    let allowed = repo_root.join("artifacts/benchmark-smoke/simulated-runs");
    anyhow::ensure!(absolute.starts_with(&allowed), "all-domain fake-run output root must resolve inside `artifacts/benchmark-smoke/simulated-runs`");
    anyhow::ensure!(
        !absolute
            .strip_prefix(repo_root)?
            .components()
            .any(|part| matches!(part, std::path::Component::ParentDir)),
        "simulation destination cannot traverse parents"
    );
    let relative = absolute.strip_prefix(repo_root)?;
    let mut current = repo_root.to_path_buf();
    for component in relative.components() {
        current.push(component);
        match fs::symlink_metadata(&current) {
            Ok(metadata) => anyhow::ensure!(
                !metadata.file_type().is_symlink(),
                "simulation destination contains symlink: {}",
                current.display()
            ),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(error).with_context(|| format!("inspect {}", current.display()))
            }
        }
    }
    Ok(absolute)
}

pub(crate) fn publish_simulation_manifest(
    path: &Path,
    manifest: &AllDomainFakeRunsReport,
) -> Result<()> {
    bijux_dna_infra::atomic_write_json(path, manifest)
        .context("publish all-domain simulation manifest")
}

pub(crate) mod failure;

pub(crate) mod missing;

pub(crate) mod completion;
mod paths;

pub(crate) mod parser;
