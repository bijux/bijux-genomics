//! Semantic admission for persisted local HPC dependency simulations.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::{Component, Path};

use crate::{
    BenchError, BenchmarkLocalHpcDependencySimulationCase,
    BenchmarkLocalHpcDependencySimulationJob, BenchmarkLocalHpcDependencySimulationReport,
    BENCHMARK_LOCAL_HPC_DEPENDENCY_SIMULATION_SCHEMA_V1,
    BENCHMARK_LOCAL_HPC_REFERENCE_BLOCKED_JOB_ID, BENCHMARK_LOCAL_HPC_REFERENCE_CONTINUED_JOB_ID,
    BENCHMARK_LOCAL_HPC_REFERENCE_FAILURE_JOB_ID, BENCHMARK_LOCAL_HPC_RELATEDNESS_BLOCKED_JOB_ID,
    BENCHMARK_LOCAL_HPC_RELATEDNESS_CONTINUED_JOB_ID,
    BENCHMARK_LOCAL_HPC_RELATEDNESS_FAILURE_JOB_ID,
};

/// Validate dependency topology, branch isolation, timing, counters, and seeded proofs.
///
/// # Errors
/// Returns an error when a persisted dependency simulation is inconsistent.
pub fn validate_benchmark_local_hpc_dependency_simulation(
    report: &BenchmarkLocalHpcDependencySimulationReport,
) -> Result<(), BenchError> {
    if report.schema_version != BENCHMARK_LOCAL_HPC_DEPENDENCY_SIMULATION_SCHEMA_V1 {
        return invalid("unsupported local HPC dependency-simulation schema");
    }
    require_path(&report.manifest_path, "manifest_path")?;
    require_path(&report.output_path, "output_path")?;
    if report.job_count == 0
        || report.job_count != report.benchmark_job_count + report.essential_pipeline_job_count
        || report.case_count != report.cases.len()
        || report.case_count != 2
        || !report.all_cases_proven
        || !report.cases.iter().all(|case| case.proven)
    {
        return invalid("local HPC dependency-simulation aggregate does not close");
    }
    let expected = [
        (
            "relatedness_ibd_failure_isolates_demography_from_roh",
            BENCHMARK_LOCAL_HPC_RELATEDNESS_FAILURE_JOB_ID,
            BENCHMARK_LOCAL_HPC_RELATEDNESS_BLOCKED_JOB_ID,
            BENCHMARK_LOCAL_HPC_RELATEDNESS_CONTINUED_JOB_ID,
        ),
        (
            "reference_panel_prepare_failure_blocks_phasing_without_blocking_qc",
            BENCHMARK_LOCAL_HPC_REFERENCE_FAILURE_JOB_ID,
            BENCHMARK_LOCAL_HPC_REFERENCE_BLOCKED_JOB_ID,
            BENCHMARK_LOCAL_HPC_REFERENCE_CONTINUED_JOB_ID,
        ),
    ];
    for (case, proof) in report.cases.iter().zip(expected) {
        if case.case_id != proof.0
            || case.failed_job_id != proof.1
            || case.blocked_descendant_job_id != proof.2
            || case.continued_sibling_job_id != proof.3
        {
            return invalid("local HPC dependency-simulation seeded proof identity drifted");
        }
        validate_case(report, case)?;
    }
    let first_scope = report.cases[0]
        .jobs
        .iter()
        .map(|job| {
            (
                &job.job_id_local,
                &job.job_kind,
                &job.result_id,
                &job.pipeline_id,
                &job.node_id,
                &job.stage_id,
                &job.dependencies,
                &job.dependents,
            )
        })
        .collect::<Vec<_>>();
    let second_scope = report.cases[1]
        .jobs
        .iter()
        .map(|job| {
            (
                &job.job_id_local,
                &job.job_kind,
                &job.result_id,
                &job.pipeline_id,
                &job.node_id,
                &job.stage_id,
                &job.dependencies,
                &job.dependents,
            )
        })
        .collect::<Vec<_>>();
    if first_scope != second_scope {
        return invalid("local HPC dependency-simulation case scopes diverged");
    }
    Ok(())
}

fn validate_case(
    report: &BenchmarkLocalHpcDependencySimulationReport,
    case: &BenchmarkLocalHpcDependencySimulationCase,
) -> Result<(), BenchError> {
    require_text(&case.case_id, "cases[].case_id")?;
    if case.jobs.len() != report.job_count {
        return invalid("local HPC dependency-simulation case job count drifted");
    }
    let by_id =
        case.jobs.iter().map(|job| (job.job_id_local.as_str(), job)).collect::<BTreeMap<_, _>>();
    if by_id.len() != case.jobs.len() || !by_id.contains_key(case.failed_job_id.as_str()) {
        return invalid("local HPC dependency-simulation repeats or omits a job identity");
    }
    let mut prior = BTreeSet::new();
    let mut dependency_count = 0usize;
    for job in &case.jobs {
        validate_job(job, &by_id, &prior, &case.failed_job_id)?;
        dependency_count += job.dependencies.len();
        prior.insert(job.job_id_local.as_str());
    }
    let benchmark_count = case.jobs.iter().filter(|job| job.job_kind == "benchmark_result").count();
    if benchmark_count != report.benchmark_job_count
        || case.jobs.len() - benchmark_count != report.essential_pipeline_job_count
        || dependency_count != report.dependency_count
    {
        return invalid("local HPC dependency-simulation case does not match report scope");
    }
    let descendants = descendants(&by_id, &case.failed_job_id)?;
    let blocked = case
        .jobs
        .iter()
        .filter(|job| job.status == "blocked")
        .map(|job| job.job_id_local.clone())
        .collect::<Vec<_>>();
    let continued = case
        .jobs
        .iter()
        .filter(|job| job.status == "completed" && job.unrelated_to_failed_job)
        .map(|job| job.job_id_local.clone())
        .collect::<Vec<_>>();
    let continued_benchmarks = case
        .jobs
        .iter()
        .filter(|job| {
            job.status == "completed"
                && job.unrelated_to_failed_job
                && job.job_kind == "benchmark_result"
        })
        .map(|job| job.job_id_local.clone())
        .collect::<Vec<_>>();
    let makespan = case.jobs.iter().map(|job| job.finish_second).max().unwrap_or(0);
    let descendant_set = descendants.iter().collect::<BTreeSet<_>>();
    let flags_close = case.jobs.iter().all(|job| {
        let descendant = descendant_set.contains(&job.job_id_local);
        job.descendant_of_failed_job == descendant
            && job.unrelated_to_failed_job
                == (job.job_id_local != case.failed_job_id && !descendant)
    });
    if case.descendant_job_ids != descendants
        || case.blocked_job_ids != blocked
        || case.continued_unrelated_job_ids != continued
        || case.continued_benchmark_job_ids != continued_benchmarks
        || case.simulated_makespan_seconds != makespan
        || !flags_close
        || case.blocked_only_descendants
            != case.blocked_job_ids.iter().all(|id| case.descendant_job_ids.contains(id))
        || case.unrelated_branches_continue
            != case.continued_unrelated_job_ids.contains(&case.continued_sibling_job_id)
        || case.benchmark_jobs_continue == case.continued_benchmark_job_ids.is_empty()
        || !case.blocked_only_descendants
        || !case.unrelated_branches_continue
        || !case.benchmark_jobs_continue
        || !case.blocked_job_ids.contains(&case.blocked_descendant_job_id)
    {
        return invalid("local HPC dependency-simulation branch proof does not close");
    }
    Ok(())
}

fn validate_job<'a>(
    job: &'a BenchmarkLocalHpcDependencySimulationJob,
    by_id: &BTreeMap<&str, &'a BenchmarkLocalHpcDependencySimulationJob>,
    prior: &BTreeSet<&str>,
    failed_job_id: &str,
) -> Result<(), BenchError> {
    for (value, field) in [
        (&job.job_id_local, "jobs[].job_id_local"),
        (&job.job_kind, "jobs[].job_kind"),
        (&job.stage_id, "jobs[].stage_id"),
        (&job.status, "jobs[].status"),
    ] {
        require_text(value, field)?;
    }
    let identity_valid = match job.job_kind.as_str() {
        "benchmark_result" => {
            job.result_id.is_some() && job.pipeline_id.is_none() && job.node_id.is_none()
        }
        "essential_pipeline_node" => {
            job.result_id.is_none() && job.pipeline_id.is_some() && job.node_id.is_some()
        }
        _ => false,
    };
    if !identity_valid
        || job.dependency_count != job.dependencies.len()
        || job.dependent_count != job.dependents.len()
    {
        return invalid("local HPC dependency-simulation job identity does not close");
    }
    for dependency in &job.dependencies {
        let Some(parent) = by_id.get(dependency.as_str()) else {
            return invalid("local HPC dependency simulation references an unknown dependency");
        };
        if !prior.contains(dependency.as_str()) || !parent.dependents.contains(&job.job_id_local) {
            return invalid("local HPC dependency simulation is not topological or reciprocal");
        }
    }
    for dependent in &job.dependents {
        let Some(child) = by_id.get(dependent.as_str()) else {
            return invalid("local HPC dependency simulation references an unknown dependent");
        };
        if !child.dependencies.contains(&job.job_id_local) {
            return invalid("local HPC dependency simulation dependent edge is not reciprocal");
        }
    }
    let blocking = job
        .dependencies
        .iter()
        .filter(|dependency| {
            by_id
                .get(dependency.as_str())
                .is_some_and(|parent| matches!(parent.status.as_str(), "failed" | "blocked"))
        })
        .cloned()
        .collect::<Vec<_>>();
    let expected_status = if job.job_id_local == failed_job_id {
        "failed"
    } else if blocking.is_empty() {
        "completed"
    } else {
        "blocked"
    };
    let timing_valid = if expected_status == "blocked" {
        job.start_second == 0 && job.finish_second == 0 && job.duration_seconds == 0
    } else {
        let dependency_finish = job
            .dependencies
            .iter()
            .filter_map(|dependency| by_id.get(dependency.as_str()))
            .map(|parent| parent.finish_second)
            .max()
            .unwrap_or(0);
        job.start_second == dependency_finish
            && job.duration_seconds == 1
            && job.finish_second == job.start_second + 1
    };
    if job.blocking_dependencies != blocking || job.status != expected_status || !timing_valid {
        return invalid("local HPC dependency-simulation status or timing is inconsistent");
    }
    Ok(())
}

fn descendants(
    by_id: &BTreeMap<&str, &BenchmarkLocalHpcDependencySimulationJob>,
    failed_job_id: &str,
) -> Result<Vec<String>, BenchError> {
    let mut queue = by_id
        .get(failed_job_id)
        .ok_or_else(|| BenchError::InvalidPolicy("missing failed job".to_string()))?
        .dependents
        .iter()
        .cloned()
        .collect::<VecDeque<_>>();
    let mut seen = BTreeSet::new();
    let mut found = Vec::new();
    while let Some(id) = queue.pop_front() {
        if !seen.insert(id.clone()) {
            continue;
        }
        let job = by_id
            .get(id.as_str())
            .ok_or_else(|| BenchError::InvalidPolicy("missing descendant job".to_string()))?;
        found.push(id);
        queue.extend(job.dependents.iter().cloned());
    }
    Ok(found)
}

fn require_text(value: &str, field: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() {
        return invalid(format!("local HPC dependency simulation requires non-empty {field}"));
    }
    Ok(())
}

fn require_path(value: &str, field: &str) -> Result<(), BenchError> {
    require_text(value, field)?;
    let path = Path::new(value);
    if path.is_absolute()
        || path.components().any(|part| {
            matches!(
                part,
                Component::Prefix(_)
                    | Component::RootDir
                    | Component::ParentDir
                    | Component::CurDir
            )
        })
    {
        return invalid(format!(
            "local HPC dependency simulation requires safe repository-relative {field}"
        ));
    }
    Ok(())
}

fn invalid<T>(message: impl Into<String>) -> Result<T, BenchError> {
    Err(BenchError::InvalidPolicy(message.into()))
}
