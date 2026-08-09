//! Thin operator composition for generic benchmark case evidence.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result};

use super::{require_exact_options, required, resolve_path, OpsCommandOutcome, Workspace};

const MAX_CASE_INPUT_BYTES: usize = 64 * 1024 * 1024;

pub(super) fn plan_cases(
    workspace: &Workspace,
    options: &BTreeMap<String, String>,
) -> Result<OpsCommandOutcome> {
    require_exact_options(options, &["--corpus-snapshots", "--output", "--repository-root"])?;
    let repository_root = resolve_path(workspace, required(options, "--repository-root")?)
        .canonicalize()
        .context("canonicalize generic benchmark source repository")?;
    let repository_commit =
        bijux_dna_containers::api::clean_repository_head_commit(&repository_root)
            .context("require a clean generic benchmark source repository")?;
    let campaign_path = repository_root.join("benchmarks/contracts/stage-campaigns.toml");
    let interface_path = repository_root.join("benchmarks/contracts/stage-interfaces.toml");
    let campaigns = bijux_dna_bench::load_stage_campaign_registry(&campaign_path)?;
    let interfaces =
        bijux_dna_bench::load_stage_interface_catalog_from_repo(&interface_path, &repository_root)?;
    let bindings = bijux_dna_stage_contract::canonical_tool_bindings()?;
    let corpus_snapshots: Vec<bijux_dna_bench::BenchmarkCorpusSnapshotV1> = read_json(
        &resolve_path(workspace, required(options, "--corpus-snapshots")?),
        "benchmark corpus snapshots",
    )?;
    let binding_bytes = serde_json::to_vec(&bindings.bindings().cloned().collect::<Vec<_>>())?;
    let matrix = bijux_dna_bench::plan_benchmark_case_matrix(
        &bijux_dna_bench::BenchmarkCaseMatrixRequest {
            repository_commit: &repository_commit,
            campaign_registry_sha256: &bijux_dna_infra::hash_file_sha256(&campaign_path)?,
            interface_catalog_sha256: &bijux_dna_infra::hash_file_sha256(&interface_path)?,
            tool_binding_registry_sha256: &bijux_dna_infra::hash_bytes_sha256(&binding_bytes),
            campaigns: &campaigns,
            interfaces: &interfaces,
            bindings: &bindings,
            corpus_snapshots: &corpus_snapshots,
        },
    )?;
    let output = resolve_path(workspace, required(options, "--output")?);
    bijux_dna_bench::ensure_benchmark_case_matrix(&output, &matrix)?;
    Ok(OpsCommandOutcome::success(format!(
        "planned {} generic benchmark cases\nrepository_commit\t{}\ncase_matrix_sha256\t{}\noutput\t{}\n",
        matrix.cases.len(),
        matrix.repository_commit,
        bijux_dna_bench::benchmark_case_matrix_sha256(&matrix)?,
        output.display(),
    )))
}

pub(super) fn admit_cases(
    workspace: &Workspace,
    options: &BTreeMap<String, String>,
) -> Result<OpsCommandOutcome> {
    require_exact_options(options, &["--matrix", "--observations", "--output"])?;
    let matrix = bijux_dna_bench::load_benchmark_case_matrix(&resolve_path(
        workspace,
        required(options, "--matrix")?,
    ))?;
    let observations: Vec<bijux_dna_bench::BenchmarkAdmittedCaseObservationV2> = read_json(
        &resolve_path(workspace, required(options, "--observations")?),
        "admitted benchmark case observations",
    )?;
    let set =
        bijux_dna_bench::build_benchmark_admitted_case_observation_set(&matrix, observations)?;
    let output = resolve_path(workspace, required(options, "--output")?);
    bijux_dna_bench::ensure_benchmark_admitted_case_observation_set(&output, &matrix, &set)?;
    Ok(OpsCommandOutcome::success(format!(
        "admitted {} generic benchmark case observations\ncase_matrix_sha256\t{}\noutput\t{}\n",
        set.observations.len(),
        set.case_matrix_sha256,
        output.display(),
    )))
}

pub(super) fn aggregate_cases(
    workspace: &Workspace,
    options: &BTreeMap<String, String>,
) -> Result<OpsCommandOutcome> {
    require_exact_options(options, &["--matrix", "--observations", "--output"])?;
    let matrix = bijux_dna_bench::load_benchmark_case_matrix(&resolve_path(
        workspace,
        required(options, "--matrix")?,
    ))?;
    let observations = bijux_dna_bench::load_benchmark_admitted_case_observation_set(
        &resolve_path(workspace, required(options, "--observations")?),
        &matrix,
    )?;
    let aggregations =
        bijux_dna_bench::aggregate_benchmark_admitted_case_observation_set(&matrix, &observations)?;
    let output = resolve_path(workspace, required(options, "--output")?);
    bijux_dna_bench::ensure_benchmark_case_aggregation_set(
        &output,
        &matrix,
        &observations,
        &aggregations,
    )?;
    Ok(OpsCommandOutcome::success(format!(
        "aggregated {} generic benchmark cases\nobservation_set_sha256\t{}\noutput\t{}\n",
        aggregations.aggregations.len(),
        aggregations.observation_set_sha256,
        output.display(),
    )))
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path, label: &str) -> Result<T> {
    let bytes = bijux_dna_infra::read_to_end_bounded(path, MAX_CASE_INPUT_BYTES)
        .with_context(|| format!("read {label} {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("parse {label} {}", path.display()))
}
