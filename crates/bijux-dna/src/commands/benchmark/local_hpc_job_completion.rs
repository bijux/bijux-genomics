use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

use super::local_all_domain_slurm_submit_manifest::BenchLocalAllDomainSlurmSubmitJob;
use super::local_stage_result_manifest::{
    load_validated_stage_result_manifest_path, BenchStageResultStatus,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LocalHpcJobCompletionState {
    ValidCompleted,
    FailedStageResultManifest,
    InvalidStageResultManifest,
    MissingStageResultManifest,
    StalePartialOutputs,
}

impl LocalHpcJobCompletionState {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::ValidCompleted => "valid_completed",
            Self::FailedStageResultManifest => "failed_stage_result_manifest",
            Self::InvalidStageResultManifest => "invalid_stage_result_manifest",
            Self::MissingStageResultManifest => "missing_stage_result_manifest",
            Self::StalePartialOutputs => "stale_partial_outputs",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LocalHpcJobResultPaths {
    pub(crate) result_root: PathBuf,
    pub(crate) stage_result_manifest_path: PathBuf,
    pub(crate) declared_output_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LocalHpcJobCompletionClassification {
    pub(crate) state: LocalHpcJobCompletionState,
    pub(crate) detail: String,
    pub(crate) manifest_valid: bool,
    pub(crate) runtime_status: Option<String>,
    pub(crate) declared_output_count: usize,
    pub(crate) present_output_count: usize,
    pub(crate) missing_output_paths: Vec<String>,
}

pub(crate) fn resolve_local_hpc_job_result_paths(
    repo_root: &Path,
    job: &BenchLocalAllDomainSlurmSubmitJob,
) -> Result<LocalHpcJobResultPaths> {
    let stdout_path = resolve_repo_relative_path(repo_root, Path::new(&job.stdout));
    let stderr_path = resolve_repo_relative_path(repo_root, Path::new(&job.stderr));
    let result_root = stdout_path.parent().map(Path::to_path_buf).ok_or_else(|| {
        anyhow!(
            "HPC job `{}` stdout path `{}` has no parent directory",
            job.job_id_local,
            stdout_path.display()
        )
    })?;
    let stderr_root = stderr_path.parent().map(Path::to_path_buf).ok_or_else(|| {
        anyhow!(
            "HPC job `{}` stderr path `{}` has no parent directory",
            job.job_id_local,
            stderr_path.display()
        )
    })?;
    if result_root != stderr_root {
        return Err(anyhow!(
            "HPC job `{}` stdout root `{}` and stderr root `{}` diverged",
            job.job_id_local,
            result_root.display(),
            stderr_root.display()
        ));
    }

    let default_stage_result_manifest_path = result_root.join("stage-result.json");
    let stage_result_manifest_path = job
        .outputs
        .iter()
        .map(|path| resolve_repo_relative_path(repo_root, Path::new(path)))
        .find(|path| *path == default_stage_result_manifest_path)
        .unwrap_or_else(|| default_stage_result_manifest_path.clone());

    let declared_output_paths = job
        .outputs
        .iter()
        .map(|path| resolve_repo_relative_path(repo_root, Path::new(path)))
        .filter(|path| *path != stage_result_manifest_path)
        .collect::<Vec<_>>();

    Ok(LocalHpcJobResultPaths {
        result_root,
        stage_result_manifest_path,
        declared_output_paths,
    })
}

pub(crate) fn classify_local_hpc_job_completion(
    repo_root: &Path,
    job: &BenchLocalAllDomainSlurmSubmitJob,
    result_paths: &LocalHpcJobResultPaths,
) -> LocalHpcJobCompletionClassification {
    if !result_paths.stage_result_manifest_path.is_file() {
        return LocalHpcJobCompletionClassification {
            state: LocalHpcJobCompletionState::MissingStageResultManifest,
            detail: "missing_stage_result_manifest".to_string(),
            manifest_valid: false,
            runtime_status: None,
            declared_output_count: result_paths.declared_output_paths.len(),
            present_output_count: 0,
            missing_output_paths: result_paths
                .declared_output_paths
                .iter()
                .map(|path| path_relative_to_repo(repo_root, path))
                .collect(),
        };
    }

    match load_validated_stage_result_manifest_path(&result_paths.stage_result_manifest_path) {
        Ok(manifest) => {
            if manifest.stage_id != job.stage_id || manifest.tool.id != job.tool_id {
                return LocalHpcJobCompletionClassification {
                    state: LocalHpcJobCompletionState::InvalidStageResultManifest,
                    detail: "identity_mismatch".to_string(),
                    manifest_valid: false,
                    runtime_status: Some(match manifest.runtime.status {
                        BenchStageResultStatus::Succeeded => "succeeded".to_string(),
                        BenchStageResultStatus::Failed => "failed".to_string(),
                    }),
                    declared_output_count: result_paths.declared_output_paths.len(),
                    present_output_count: 0,
                    missing_output_paths: result_paths
                        .declared_output_paths
                        .iter()
                        .map(|path| path_relative_to_repo(repo_root, path))
                        .collect(),
                };
            }

            let runtime_status = match manifest.runtime.status {
                BenchStageResultStatus::Succeeded => "succeeded".to_string(),
                BenchStageResultStatus::Failed => "failed".to_string(),
            };
            if manifest.runtime.status == BenchStageResultStatus::Failed {
                return LocalHpcJobCompletionClassification {
                    state: LocalHpcJobCompletionState::FailedStageResultManifest,
                    detail: "runtime_status_failed".to_string(),
                    manifest_valid: true,
                    runtime_status: Some(runtime_status),
                    declared_output_count: result_paths.declared_output_paths.len(),
                    present_output_count: 0,
                    missing_output_paths: result_paths
                        .declared_output_paths
                        .iter()
                        .map(|path| path_relative_to_repo(repo_root, path))
                        .collect(),
                };
            }

            let manifest_output_paths = manifest
                .outputs
                .iter()
                .map(|output| resolve_repo_relative_path(repo_root, Path::new(&output.realized_path)))
                .collect::<Vec<_>>();
            if manifest_output_paths.len() != result_paths.declared_output_paths.len() {
                return LocalHpcJobCompletionClassification {
                    state: LocalHpcJobCompletionState::StalePartialOutputs,
                    detail: "manifest_output_count_mismatch".to_string(),
                    manifest_valid: true,
                    runtime_status: Some(runtime_status),
                    declared_output_count: result_paths.declared_output_paths.len(),
                    present_output_count: 0,
                    missing_output_paths: result_paths
                        .declared_output_paths
                        .iter()
                        .map(|path| path_relative_to_repo(repo_root, path))
                        .collect(),
                };
            }

            let missing_output_paths = manifest
                .outputs
                .iter()
                .zip(result_paths.declared_output_paths.iter())
                .filter_map(|(output, expected_path)| {
                    let realized_path =
                        resolve_repo_relative_path(repo_root, Path::new(&output.realized_path));
                    if realized_path != *expected_path || !output.exists || !realized_path.exists() {
                        Some(path_relative_to_repo(repo_root, expected_path))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            let present_output_count =
                result_paths.declared_output_paths.len().saturating_sub(missing_output_paths.len());
            if !missing_output_paths.is_empty() {
                return LocalHpcJobCompletionClassification {
                    state: LocalHpcJobCompletionState::StalePartialOutputs,
                    detail: "missing_declared_outputs".to_string(),
                    manifest_valid: true,
                    runtime_status: Some(runtime_status),
                    declared_output_count: result_paths.declared_output_paths.len(),
                    present_output_count,
                    missing_output_paths,
                };
            }

            LocalHpcJobCompletionClassification {
                state: LocalHpcJobCompletionState::ValidCompleted,
                detail: "valid_completed".to_string(),
                manifest_valid: true,
                runtime_status: Some(runtime_status),
                declared_output_count: result_paths.declared_output_paths.len(),
                present_output_count: result_paths.declared_output_paths.len(),
                missing_output_paths: Vec::new(),
            }
        }
        Err(err) => LocalHpcJobCompletionClassification {
            state: LocalHpcJobCompletionState::InvalidStageResultManifest,
            detail: format!("{err:#}"),
            manifest_valid: false,
            runtime_status: None,
            declared_output_count: result_paths.declared_output_paths.len(),
            present_output_count: 0,
            missing_output_paths: result_paths
                .declared_output_paths
                .iter()
                .map(|path| path_relative_to_repo(repo_root, path))
                .collect(),
        },
    }
}

fn resolve_repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use tempfile::tempdir;

    use super::{
        classify_local_hpc_job_completion, resolve_local_hpc_job_result_paths,
        LocalHpcJobCompletionState,
    };
    use crate::commands::benchmark::local_all_domain_slurm_submit_manifest::{
        BenchLocalAllDomainSlurmSubmitJob, BenchLocalAllDomainSlurmSubmitResources,
    };
    use crate::commands::benchmark::local_stage_result_manifest::{
        validate_stage_result_manifest, BenchStageResultCommandV1, BenchStageResultManifestV1,
        BenchStageResultOutputV1, BenchStageResultResourceMetricSource,
        BenchStageResultResourceMetricsV1, BenchStageResultRuntimeV1, BenchStageResultStatus,
        BenchStageResultToolV1, BENCH_STAGE_RESULT_SCHEMA_VERSION,
    };

    fn benchmark_job() -> BenchLocalAllDomainSlurmSubmitJob {
        BenchLocalAllDomainSlurmSubmitJob {
            job_id_local: "benchmark:vcf:vcf_production_regression:vcf.stats:vcf_cohort:bcftools"
                .to_string(),
            domain: "vcf".to_string(),
            stage_id: "vcf.stats".to_string(),
            pipeline_id: None,
            node_id: None,
            tool_id: "bcftools".to_string(),
            corpus_id: "vcf_production_regression".to_string(),
            asset_profile_id: "vcf_cohort".to_string(),
            result_id: Some(
                "vcf:vcf_production_regression:vcf.stats:vcf_cohort:bcftools".to_string(),
            ),
            script_path: "runs/bench/slurm-dry-run/all-domains/controls/job.sbatch".to_string(),
            stdout: "runs/bench/slurm-dry-run/all-domains/runs/all-domain-benchmark-dry-run/vcf/vcf.stats/bcftools/vcf_production_regression/vcf_cohort/stdout.log".to_string(),
            stderr: "runs/bench/slurm-dry-run/all-domains/runs/all-domain-benchmark-dry-run/vcf/vcf.stats/bcftools/vcf_production_regression/vcf_cohort/stderr.log".to_string(),
            outputs: vec![
                "runs/bench/slurm-dry-run/all-domains/runs/all-domain-benchmark-dry-run/vcf/vcf.stats/bcftools/vcf_production_regression/vcf_cohort/declared-outputs/bcftools_stats_txt.txt".to_string(),
                "runs/bench/slurm-dry-run/all-domains/runs/all-domain-benchmark-dry-run/vcf/vcf.stats/bcftools/vcf_production_regression/vcf_cohort/declared-outputs/stats.json".to_string(),
                "runs/bench/slurm-dry-run/all-domains/runs/all-domain-benchmark-dry-run/vcf/vcf.stats/bcftools/vcf_production_regression/vcf_cohort/stage-result.json".to_string(),
            ],
            dependencies: Vec::new(),
            resources: BenchLocalAllDomainSlurmSubmitResources {
                cpus_per_task: 1,
                memory_mb: 1024,
                time_limit: "00:10:00".to_string(),
            },
        }
    }

    fn write_stage_result(
        repo_root: &Path,
        job: &BenchLocalAllDomainSlurmSubmitJob,
        output_exists: bool,
        status: BenchStageResultStatus,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let result_paths = resolve_local_hpc_job_result_paths(repo_root, job)?;
        fs::create_dir_all(&result_paths.result_root)?;
        fs::write(result_paths.result_root.join("stdout.log"), "stdout\n")?;
        fs::write(result_paths.result_root.join("stderr.log"), "stderr\n")?;
        for output_path in &result_paths.declared_output_paths {
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)?;
            }
            if output_exists {
                fs::write(output_path, "payload\n")?;
            }
        }
        let manifest = BenchStageResultManifestV1 {
            schema_version: BENCH_STAGE_RESULT_SCHEMA_VERSION.to_string(),
            stage_id: job.stage_id.clone(),
            tool: BenchStageResultToolV1 { id: job.tool_id.clone() },
            command: BenchStageResultCommandV1 { rendered: "echo run".to_string() },
            runtime: BenchStageResultRuntimeV1 {
                mode: "hpc_resume_simulation".to_string(),
                status,
                started_at: "1970-01-01T00:00:00Z".to_string(),
                finished_at: "1970-01-01T00:00:01Z".to_string(),
                elapsed_seconds: 1.0,
                exit_code: 0,
            },
            resource_metrics: BenchStageResultResourceMetricsV1 {
                source: BenchStageResultResourceMetricSource::NotAvailable,
                memory_mb: None,
                cpu_threads: None,
            },
            outputs: result_paths
                .declared_output_paths
                .iter()
                .map(|path| BenchStageResultOutputV1 {
                    artifact_id: path
                        .file_stem()
                        .and_then(|value| value.to_str())
                        .unwrap_or("artifact")
                        .to_string(),
                    declared_path: path_relative_to_repo(repo_root, path),
                    realized_path: path_relative_to_repo(repo_root, path),
                    role: "declared_output".to_string(),
                    optional: false,
                    exists: output_exists,
                })
                .collect(),
        };
        validate_stage_result_manifest(&manifest)?;
        bijux_dna_infra::atomic_write_json(&result_paths.stage_result_manifest_path, &manifest)?;
        Ok(())
    }

    fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
        path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
    }

    #[test]
    fn resolve_local_hpc_job_result_paths_excludes_stage_result_from_declared_outputs() {
        let repo_root = tempdir().expect("tempdir");
        let paths =
            resolve_local_hpc_job_result_paths(repo_root.path(), &benchmark_job()).expect("paths");
        assert!(paths.stage_result_manifest_path.ends_with("stage-result.json"));
        assert_eq!(paths.declared_output_paths.len(), 2);
        assert!(paths
            .declared_output_paths
            .iter()
            .all(|path| path.file_name().and_then(|value| value.to_str()) != Some("stage-result.json")));
    }

    #[test]
    fn classify_local_hpc_job_completion_accepts_valid_completed_outputs() {
        let repo_root = tempdir().expect("tempdir");
        let job = benchmark_job();
        write_stage_result(repo_root.path(), &job, true, BenchStageResultStatus::Succeeded)
            .expect("write stage result");
        let result_paths =
            resolve_local_hpc_job_result_paths(repo_root.path(), &job).expect("result paths");
        let classification =
            classify_local_hpc_job_completion(repo_root.path(), &job, &result_paths);
        assert_eq!(classification.state, LocalHpcJobCompletionState::ValidCompleted);
        assert_eq!(classification.present_output_count, 2);
    }

    #[test]
    fn classify_local_hpc_job_completion_marks_failed_manifests_for_rerun() {
        let repo_root = tempdir().expect("tempdir");
        let job = benchmark_job();
        write_stage_result(repo_root.path(), &job, true, BenchStageResultStatus::Failed)
            .expect("write stage result");
        let result_paths =
            resolve_local_hpc_job_result_paths(repo_root.path(), &job).expect("result paths");
        let classification =
            classify_local_hpc_job_completion(repo_root.path(), &job, &result_paths);
        assert_eq!(classification.state, LocalHpcJobCompletionState::FailedStageResultManifest);
    }

    #[test]
    fn classify_local_hpc_job_completion_marks_missing_manifest_for_rerun() {
        let repo_root = tempdir().expect("tempdir");
        let job = benchmark_job();
        let result_paths =
            resolve_local_hpc_job_result_paths(repo_root.path(), &job).expect("result paths");
        fs::create_dir_all(&result_paths.result_root).expect("result root");
        let classification =
            classify_local_hpc_job_completion(repo_root.path(), &job, &result_paths);
        assert_eq!(classification.state, LocalHpcJobCompletionState::MissingStageResultManifest);
    }

    #[test]
    fn classify_local_hpc_job_completion_rejects_missing_declared_outputs() {
        let repo_root = tempdir().expect("tempdir");
        let job = benchmark_job();
        write_stage_result(repo_root.path(), &job, true, BenchStageResultStatus::Succeeded)
            .expect("write stage result");
        let result_paths =
            resolve_local_hpc_job_result_paths(repo_root.path(), &job).expect("result paths");
        fs::remove_file(&result_paths.declared_output_paths[0]).expect("remove output");
        let classification =
            classify_local_hpc_job_completion(repo_root.path(), &job, &result_paths);
        assert_eq!(classification.state, LocalHpcJobCompletionState::StalePartialOutputs);
        assert_eq!(classification.present_output_count, 1);
        assert_eq!(classification.missing_output_paths.len(), 1);
    }
}
