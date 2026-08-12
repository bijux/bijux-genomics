//! Owner: bijux-dna-bench-model
//! Semantic admission for persisted Slurm dependency-source evidence.

use std::collections::BTreeSet;

use crate::{model::BenchmarkLocalSlurmDependencyCheckReport, BenchError};

use super::BENCHMARK_LOCAL_SLURM_DEPENDENCY_CHECK_SCHEMA_V1;

/// Validate persisted Slurm dependency-source evidence and its aggregate closure.
///
/// # Errors
/// Returns an error when identity, dependency-source policy, or aggregates drift.
pub fn validate_benchmark_local_slurm_dependency_check(
    report: &BenchmarkLocalSlurmDependencyCheckReport,
) -> Result<(), BenchError> {
    if report.schema_version != BENCHMARK_LOCAL_SLURM_DEPENDENCY_CHECK_SCHEMA_V1 {
        return invalid("unsupported local Slurm dependency-check schema");
    }
    for (field, value) in [
        ("root_path", report.root_path.as_str()),
        ("manifest_path", report.manifest_path.as_str()),
        ("report_path", report.report_path.as_str()),
    ] {
        require_text(value, field)?;
    }
    if report.jobs.is_empty()
        || report.job_count != report.jobs.len()
        || report.manifest_dependency_count
            != report.jobs.iter().map(|job| job.manifest_dependencies.len()).sum::<usize>()
        || report.script_header_dependency_count
            != report.jobs.iter().map(|job| job.script_header_dependencies.len()).sum::<usize>()
        || report.findings_count
            != report.report_findings.len()
                + report.jobs.iter().map(|job| job.findings.len()).sum::<usize>()
        || report.ok != (report.findings_count == 0)
    {
        return invalid("local Slurm dependency-check aggregates do not close");
    }
    validate_unique_text(&report.report_findings, "report_findings")?;
    let mut job_names = BTreeSet::new();
    let mut script_paths = BTreeSet::new();
    for job in &report.jobs {
        require_text(&job.job_name, "jobs[].job_name")?;
        require_text(&job.script_path, "jobs[].script_path")?;
        if !job_names.insert(job.job_name.as_str())
            || !script_paths.insert(job.script_path.as_str())
        {
            return invalid("local Slurm dependency-check repeats a job or script identity");
        }
        if let Some(stage_id) = &job.stage_id {
            require_text(stage_id, "jobs[].stage_id")?;
        }
        validate_unique_text(&job.manifest_dependencies, "jobs[].manifest_dependencies")?;
        validate_unique_text(&job.script_header_dependencies, "jobs[].script_header_dependencies")?;
        validate_unique_text(&job.findings, "jobs[].findings")?;
        let expected_source =
            match (job.manifest_dependencies.is_empty(), job.script_header_dependencies.is_empty())
            {
                (true, true) => "none",
                (false, true) => "submit_manifest",
                (true, false) => "script_header",
                (false, false) => "mixed",
            };
        if job.dependency_source != expected_source || job.ok != job.findings.is_empty() {
            return invalid("local Slurm dependency-check job verdict drifted");
        }
    }
    Ok(())
}

fn validate_unique_text(values: &[String], field: &str) -> Result<(), BenchError> {
    let mut seen = BTreeSet::new();
    for value in values {
        require_text(value, field)?;
        if !seen.insert(value.as_str()) {
            return invalid(format!("local Slurm dependency-check repeats {field}"));
        }
    }
    Ok(())
}

fn require_text(value: &str, field: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() {
        return invalid(format!("local Slurm dependency-check requires non-empty {field}"));
    }
    Ok(())
}

fn invalid<T>(message: impl Into<String>) -> Result<T, BenchError> {
    Err(BenchError::InvalidPolicy(message.into()))
}
