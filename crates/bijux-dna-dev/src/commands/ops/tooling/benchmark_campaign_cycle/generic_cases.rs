//! Thin operator composition for generic benchmark case evidence.

use std::collections::BTreeMap;

use anyhow::Result;

use super::{require_exact_options, required, resolve_path, OpsCommandOutcome, Workspace};

pub(super) fn plan_cases(
    workspace: &Workspace,
    options: &BTreeMap<String, String>,
) -> Result<OpsCommandOutcome> {
    require_exact_options(
        options,
        &["--data-index", "--output", "--repository-root", "--run-id", "--study-root"],
    )?;
    let run_id = required(options, "--run-id")?;
    let output = resolve_path(workspace, required(options, "--output")?);
    let matrix =
        bijux_dna_bench::plan_benchmark_cases(&bijux_dna_bench::BenchmarkCasePlanningRequest {
            repository_root: resolve_path(workspace, required(options, "--repository-root")?),
            study_root: resolve_path(workspace, required(options, "--study-root")?),
            data_index_path: resolve_path(workspace, required(options, "--data-index")?),
            run_id: run_id.to_string(),
            output_path: output.clone(),
        })?;
    Ok(OpsCommandOutcome::success(format!(
        "planned {} generic benchmark cases\nrun_id\t{}\nrepository_commit\t{}\ncase_matrix_sha256\t{}\noutput\t{}\n",
        matrix.cases.len(),
        run_id,
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
    let output = resolve_path(workspace, required(options, "--output")?);
    let set =
        bijux_dna_bench::admit_benchmark_cases(&bijux_dna_bench::BenchmarkCaseAdmissionRequest {
            matrix_path: resolve_path(workspace, required(options, "--matrix")?),
            observations_path: resolve_path(workspace, required(options, "--observations")?),
            output_path: output.clone(),
        })?;
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
    let output = resolve_path(workspace, required(options, "--output")?);
    let aggregations = bijux_dna_bench::aggregate_benchmark_cases(
        &bijux_dna_bench::BenchmarkCaseAggregationRequest {
            matrix_path: resolve_path(workspace, required(options, "--matrix")?),
            observations_path: resolve_path(workspace, required(options, "--observations")?),
            output_path: output.clone(),
        },
    )?;
    Ok(OpsCommandOutcome::success(format!(
        "aggregated {} generic benchmark cases\nobservation_set_sha256\t{}\noutput\t{}\n",
        aggregations.aggregations.len(),
        aggregations.observation_set_sha256,
        output.display(),
    )))
}
