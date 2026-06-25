use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::all_domain_active_scope_complete::DEFAULT_ALL_DOMAIN_ACTIVE_SCOPE_COMPLETE_PATH;
use super::operational_benchmark_ready::{
    OperationalBenchmarkReadyBlocker, DEFAULT_OPERATIONAL_BENCHMARK_READY_PATH,
};
use crate::commands::benchmark::paths::{
    validate_benchmark_paths, DEFAULT_BENCHMARK_PATHS_VALIDATE_PATH,
    DEFAULT_DISPOSABLE_ROOT_CLEANUP_PROOF_PATH,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_ALL_DOMAIN_LOCAL_OPERATIONAL_BENCHMARK_COMPLETE_PATH: &str =
    "benchmarks/readiness/all-domains/FASTQ_BAM_VCF_LOCAL_OPERATIONAL_BENCHMARK_COMPLETE.json";
const ALL_DOMAIN_LOCAL_OPERATIONAL_BENCHMARK_COMPLETE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.all_domain_local_operational_benchmark_complete.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct AllDomainLocalOperationalBenchmarkCompleteCheck {
    pub(crate) surface_id: String,
    pub(crate) output_path: String,
    pub(crate) proof_paths: Vec<String>,
    pub(crate) ok: bool,
    pub(crate) detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AllDomainLocalOperationalBenchmarkCompleteReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) checked_surface_count: usize,
    pub(crate) passed_surface_count: usize,
    pub(crate) failed_surface_count: usize,
    pub(crate) active_row_count: usize,
    pub(crate) benchmark_ready_row_count: usize,
    pub(crate) blocker_count: usize,
    pub(crate) ok: bool,
    pub(crate) checks: Vec<AllDomainLocalOperationalBenchmarkCompleteCheck>,
    pub(crate) blockers: Vec<OperationalBenchmarkReadyBlocker>,
}

pub(crate) fn run_render_all_domain_local_operational_benchmark_complete(
    args: &parse::BenchReadinessRenderAllDomainLocalOperationalBenchmarkCompleteArgs,
) -> Result<()> {
    let repo_root = crate::commands::support::workspace_root::resolve_repo_root()?;
    let report = render_all_domain_local_operational_benchmark_complete(
        &repo_root,
        args.output.clone().unwrap_or_else(|| {
            PathBuf::from(DEFAULT_ALL_DOMAIN_LOCAL_OPERATIONAL_BENCHMARK_COMPLETE_PATH)
        }),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_all_domain_local_operational_benchmark_complete(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<AllDomainLocalOperationalBenchmarkCompleteReport> {
    let absolute_output_path = repo_relative_path(repo_root, &output_path);
    if let Some(parent) = absolute_output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let mut checks = Vec::new();
    let mut blockers = BTreeSet::new();
    let cleanup_output_path = PathBuf::from(DEFAULT_DISPOSABLE_ROOT_CLEANUP_PROOF_PATH);
    let path_validation_output_path = PathBuf::from(DEFAULT_BENCHMARK_PATHS_VALIDATE_PATH);
    match (
        validate_benchmark_paths(repo_root, path_validation_output_path.clone(), true),
        load_cleanup_proof_snapshot(repo_root, &cleanup_output_path),
    ) {
        (Ok(_), Ok(report)) if report.ok => {
            checks.push(AllDomainLocalOperationalBenchmarkCompleteCheck {
                surface_id: "benchmark_paths_cleanup_proof".to_string(),
                output_path: report.output_path,
                proof_paths: vec![
                    cleanup_output_path.display().to_string(),
                    path_validation_output_path.display().to_string(),
                ],
                ok: true,
                detail: format!(
                    "deleted_root_count={}, validator_violation_count={}, readiness_snapshot_count={}",
                    report.deleted_root_count,
                    report.validator_violation_count,
                    report.validator_readiness_snapshot_count
                ),
            });
        }
        (validation_result, proof_result) => {
            let detail = match (validation_result.err(), proof_result.err()) {
                (Some(validation_error), Some(proof_error)) => {
                    format!("{validation_error}; {proof_error}")
                }
                (Some(validation_error), None) => validation_error.to_string(),
                (None, Some(proof_error)) => proof_error.to_string(),
                (None, None) => "cleanup proof artifact is not green".to_string(),
            };
            let proof_output = cleanup_output_path.display().to_string();
            checks.push(AllDomainLocalOperationalBenchmarkCompleteCheck {
                surface_id: "benchmark_paths_cleanup_proof".to_string(),
                output_path: proof_output.clone(),
                proof_paths: vec![
                    proof_output.clone(),
                    path_validation_output_path.display().to_string(),
                ],
                ok: false,
                detail: detail.clone(),
            });
            blockers.insert(global_blocker(
                "cross",
                "benchmark.paths",
                "cleanup_proof",
                "benchmark_ready",
                "benchmark_ready",
                "disposable_root_cleanup_failed",
                &proof_output,
                &detail,
            ));
        }
    }

    let active_scope_output_path = PathBuf::from(DEFAULT_ALL_DOMAIN_ACTIVE_SCOPE_COMPLETE_PATH);
    let (active_scope_snapshot, active_scope_snapshot_error) =
        match load_active_scope_snapshot(repo_root, &active_scope_output_path) {
            Ok(snapshot) => (snapshot, None),
            Err(error) => (ActiveScopeSnapshot::default(), Some(error.to_string())),
        };
    let active_row_count = active_scope_snapshot.active_row_count;
    checks.push(AllDomainLocalOperationalBenchmarkCompleteCheck {
        surface_id: "all_domain_active_scope_complete".to_string(),
        output_path: active_scope_snapshot.output_path.clone(),
        proof_paths: vec![active_scope_snapshot.output_path.clone()],
        ok: active_scope_snapshot.ok && active_scope_snapshot.failed_surface_count == 0,
        detail: if active_scope_snapshot.ok {
            format!(
                "active_row_count={}, failed_surface_count={}",
                active_scope_snapshot.active_row_count, active_scope_snapshot.failed_surface_count
            )
        } else {
            active_scope_snapshot_error.unwrap_or_else(|| {
                format!(
                    "active-scope proof is missing, blocked, or stale: active_row_count={}, failed_surface_count={}",
                    active_scope_snapshot.active_row_count,
                    active_scope_snapshot.failed_surface_count
                )
            })
        },
    });
    if !active_scope_snapshot.ok || active_scope_snapshot.failed_surface_count != 0 {
        let failed_checks = load_active_scope_failed_checks(repo_root, &active_scope_output_path)
            .unwrap_or_default();
        if failed_checks.is_empty() {
            blockers.insert(global_blocker(
                "cross",
                "benchmark.active_scope",
                "governed_surface",
                "benchmark_ready",
                "benchmark_ready",
                "active_scope_incomplete",
                &active_scope_snapshot.output_path,
                "active retained FASTQ/BAM/VCF scope is still ambiguous",
            ));
        } else {
            blockers.extend(failed_checks);
        }
    }

    let operational_output_path = PathBuf::from(DEFAULT_OPERATIONAL_BENCHMARK_READY_PATH);
    let (operational_snapshot, operational_snapshot_error) =
        match load_operational_snapshot(repo_root, &operational_output_path) {
            Ok(snapshot) => (snapshot, None),
            Err(error) => (OperationalSnapshot::default(), Some(error.to_string())),
        };
    let benchmark_ready_row_count = operational_snapshot.benchmark_ready_row_count;
    checks.push(AllDomainLocalOperationalBenchmarkCompleteCheck {
        surface_id: "operational_benchmark_ready".to_string(),
        output_path: operational_output_path.display().to_string(),
        proof_paths: vec![operational_output_path.display().to_string()],
        ok: operational_snapshot.ok
            && operational_snapshot.blocker_count == 0
            && operational_snapshot.benchmark_ready_row_count == active_row_count,
        detail: if operational_snapshot.ok {
            format!(
                "benchmark_ready_row_count={}, blocker_count={}, active_row_count={}",
                operational_snapshot.benchmark_ready_row_count,
                operational_snapshot.blocker_count,
                active_row_count
            )
        } else {
            operational_snapshot_error.unwrap_or_else(|| {
                    format!(
                        "operational benchmark readiness proof is missing, blocked, or stale: benchmark_ready_row_count={}, blocker_count={}, active_row_count={}",
                        operational_snapshot.benchmark_ready_row_count,
                        operational_snapshot.blocker_count,
                        active_row_count
                    )
                })
        },
    });
    blockers.extend(operational_snapshot.blockers);
    if (!operational_snapshot.ok
        || operational_snapshot.blocker_count != 0
        || operational_snapshot.benchmark_ready_row_count != active_row_count)
        && blockers.is_empty()
    {
        blockers.insert(global_blocker(
            "cross",
            "benchmark.operational",
            "governed_surface",
            "benchmark_ready",
            "benchmark_ready",
            "operational_readiness_failed",
            &operational_output_path.display().to_string(),
            "local operational benchmark readiness proof is missing, blocked, or no longer matches the active all-domain scope",
        ));
    }

    let checked_surface_count = checks.len();
    let passed_surface_count = checks.iter().filter(|check| check.ok).count();
    let failed_surface_count = checked_surface_count.saturating_sub(passed_surface_count);
    let blocker_count = blockers.len();
    let ok = failed_surface_count == 0 && blocker_count == 0;

    let report = AllDomainLocalOperationalBenchmarkCompleteReport {
        schema_version: ALL_DOMAIN_LOCAL_OPERATIONAL_BENCHMARK_COMPLETE_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &absolute_output_path),
        checked_surface_count,
        passed_surface_count,
        failed_surface_count,
        active_row_count,
        benchmark_ready_row_count,
        blocker_count,
        ok,
        checks,
        blockers: blockers.into_iter().collect(),
    };
    bijux_dna_infra::atomic_write_json(&absolute_output_path, &report)
        .with_context(|| format!("write {}", absolute_output_path.display()))?;
    if report.ok {
        Ok(report)
    } else {
        Err(anyhow!(
            "all-domain local operational benchmark gate failed; inspect {}",
            report.output_path
        ))
    }
}

#[derive(Default)]
struct OperationalSnapshot {
    ok: bool,
    benchmark_ready_row_count: usize,
    blocker_count: usize,
    blockers: Vec<OperationalBenchmarkReadyBlocker>,
}

#[derive(Default)]
struct CleanupProofSnapshot {
    ok: bool,
    output_path: String,
    deleted_root_count: usize,
    validator_violation_count: usize,
    validator_readiness_snapshot_count: usize,
}

#[derive(Default)]
struct ActiveScopeSnapshot {
    ok: bool,
    output_path: String,
    active_row_count: usize,
    failed_surface_count: usize,
}

fn load_operational_snapshot(repo_root: &Path, output_path: &Path) -> Result<OperationalSnapshot> {
    let value = load_json_value(repo_root, output_path)?;
    let blockers = value
        .get("blockers")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .map(json_blocker)
        .collect::<Result<Vec<_>>>()?;
    Ok(OperationalSnapshot {
        ok: value.get("ok").and_then(serde_json::Value::as_bool).unwrap_or(false),
        benchmark_ready_row_count: json_usize(&value, "benchmark_ready_row_count")?,
        blocker_count: json_usize(&value, "blocker_count")?,
        blockers,
    })
}

fn load_active_scope_snapshot(repo_root: &Path, output_path: &Path) -> Result<ActiveScopeSnapshot> {
    let value = load_json_value(repo_root, output_path)?;
    Ok(ActiveScopeSnapshot {
        ok: value.get("ok").and_then(serde_json::Value::as_bool).unwrap_or(false),
        output_path: value
            .get("output_path")
            .and_then(serde_json::Value::as_str)
            .map(std::string::ToString::to_string)
            .unwrap_or_else(|| output_path.to_string_lossy().to_string()),
        active_row_count: json_usize(&value, "active_row_count")?,
        failed_surface_count: json_usize(&value, "failed_surface_count")?,
    })
}

fn load_cleanup_proof_snapshot(
    repo_root: &Path,
    output_path: &Path,
) -> Result<CleanupProofSnapshot> {
    let value = load_json_value(repo_root, output_path)?;
    Ok(CleanupProofSnapshot {
        ok: value.get("ok").and_then(serde_json::Value::as_bool).unwrap_or(false),
        output_path: value
            .get("output_path")
            .and_then(serde_json::Value::as_str)
            .map(std::string::ToString::to_string)
            .unwrap_or_else(|| output_path.to_string_lossy().to_string()),
        deleted_root_count: json_usize(&value, "deleted_root_count")?,
        validator_violation_count: json_usize(&value, "validator_violation_count")?,
        validator_readiness_snapshot_count: json_usize(
            &value,
            "validator_readiness_snapshot_count",
        )?,
    })
}

fn load_active_scope_failed_checks(
    repo_root: &Path,
    output_path: &Path,
) -> Result<Vec<OperationalBenchmarkReadyBlocker>> {
    let value = load_json_value(repo_root, output_path)?;
    let failed_checks = value
        .get("failed_checks")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    failed_checks
        .into_iter()
        .map(|check| {
            let surface_id = json_required_str(&check, "surface_id")?;
            let blocker_path = check
                .get("proof_paths")
                .and_then(serde_json::Value::as_array)
                .and_then(|paths| paths.first())
                .and_then(serde_json::Value::as_str)
                .map(std::string::ToString::to_string)
                .unwrap_or_else(|| output_path.to_string_lossy().to_string());
            let detail = json_required_str(&check, "detail")?;
            Ok(global_blocker(
                "cross",
                &format!("benchmark.active_scope.{surface_id}"),
                "governed_surface",
                "benchmark_ready",
                "benchmark_ready",
                "active_scope_surface_failed",
                &blocker_path,
                detail,
            ))
        })
        .collect::<Result<Vec<_>>>()
}

fn json_usize(value: &serde_json::Value, key: &str) -> Result<usize> {
    usize::try_from(value.get(key).and_then(serde_json::Value::as_u64).unwrap_or(0))
        .with_context(|| format!("convert `{key}` to usize"))
}

fn load_json_value(repo_root: &Path, output_path: &Path) -> Result<serde_json::Value> {
    let absolute_path = repo_relative_path(repo_root, output_path);
    let raw = fs::read_to_string(&absolute_path)
        .with_context(|| format!("read {}", absolute_path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", absolute_path.display()))
}

fn json_blocker(value: &serde_json::Value) -> Result<OperationalBenchmarkReadyBlocker> {
    Ok(OperationalBenchmarkReadyBlocker {
        domain: json_required_str(value, "domain")?.to_string(),
        stage_id: json_required_str(value, "stage_id")?.to_string(),
        tool_id: json_required_str(value, "tool_id")?.to_string(),
        corpus_id: json_required_str(value, "corpus_id")?.to_string(),
        asset_profile_id: json_required_str(value, "asset_profile_id")?.to_string(),
        blocker_type: json_required_str(value, "blocker_type")?.to_string(),
        blocker_path: json_required_str(value, "blocker_path")?.to_string(),
        detail: json_required_str(value, "detail")?.to_string(),
    })
}

fn json_required_str<'a>(value: &'a serde_json::Value, field: &str) -> Result<&'a str> {
    value
        .get(field)
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("missing string field `{field}`"))
}

fn global_blocker(
    domain: &str,
    stage_id: &str,
    tool_id: &str,
    corpus_id: &str,
    asset_profile_id: &str,
    blocker_type: &str,
    blocker_path: &str,
    detail: &str,
) -> OperationalBenchmarkReadyBlocker {
    OperationalBenchmarkReadyBlocker {
        domain: domain.to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
        corpus_id: corpus_id.to_string(),
        asset_profile_id: asset_profile_id.to_string(),
        blocker_type: blocker_type.to_string(),
        blocker_path: blocker_path.to_string(),
        detail: detail.to_string(),
    }
}

fn repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
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
    use std::path::PathBuf;

    #[cfg(feature = "bam_downstream")]
    use super::{
        render_all_domain_local_operational_benchmark_complete,
        DEFAULT_ALL_DOMAIN_LOCAL_OPERATIONAL_BENCHMARK_COMPLETE_PATH,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[cfg(feature = "bam_downstream")]
    #[test]
    fn all_domain_local_operational_benchmark_complete_reports_green_surface() {
        let root = repo_root();
        let report = render_all_domain_local_operational_benchmark_complete(
            &root,
            PathBuf::from(DEFAULT_ALL_DOMAIN_LOCAL_OPERATIONAL_BENCHMARK_COMPLETE_PATH),
        )
        .expect("render final local operational benchmark gate");
        assert!(report.ok);
        assert_eq!(
            report.schema_version,
            "bijux.bench.readiness.all_domain_local_operational_benchmark_complete.v1"
        );
        assert_eq!(
            report.output_path,
            "benchmarks/readiness/all-domains/FASTQ_BAM_VCF_LOCAL_OPERATIONAL_BENCHMARK_COMPLETE.json"
        );
        assert_eq!(report.checked_surface_count, 3);
        assert_eq!(report.failed_surface_count, 0);
        assert_eq!(report.active_row_count, 141);
        assert_eq!(report.benchmark_ready_row_count, 141);
        assert_eq!(report.blocker_count, 0);
    }
}
