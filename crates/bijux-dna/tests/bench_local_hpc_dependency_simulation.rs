#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn render_path(repo_root: &Path, label: &str) -> (tempfile::TempDir, PathBuf) {
    let temp_dir = tempfile::Builder::new()
        .prefix(label)
        .tempdir_in(repo_root.join("runs/bench/hpc-dry-run"))
        .expect("temporary HPC dry-run directory");
    let report_path = temp_dir.path().join("slurm-dependency-simulation.json");
    (temp_dir, report_path)
}

fn run_cli(args: &[&str]) -> std::process::Output {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(args)
        .output()
        .expect("run cli")
}

fn job_by_id<'a>(jobs: &'a [serde_json::Value], job_id: &str) -> &'a serde_json::Value {
    jobs.iter()
        .find(|job| job.get("job_id_local").and_then(serde_json::Value::as_str) == Some(job_id))
        .expect("governed job")
}

#[test]
fn bench_local_render_hpc_dependency_simulation_proves_branch_isolation() {
    let repo_root = support::repo_root().expect("repo root");
    let (_temp_dir, report_path) = render_path(&repo_root, "render-hpc-dependency-simulation-");
    let report_arg = report_path.to_string_lossy().into_owned();

    let output =
        run_cli(&["bench", "local", "render-hpc-dependency-simulation", "--output", &report_arg]);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let printed_path = String::from_utf8(output.stdout).expect("stdout utf8");
    assert_eq!(
        printed_path.trim(),
        report_path
            .strip_prefix(&repo_root)
            .expect("report path relative to repo root")
            .to_string_lossy()
    );

    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(&report_path).expect("read report"))
            .expect("parse report");
    assert_eq!(
        report.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_hpc_dependency_simulation.v1")
    );
    assert_eq!(
        report.get("output_path").and_then(serde_json::Value::as_str),
        Some(
            report_path
                .strip_prefix(&repo_root)
                .expect("report path relative to repo root")
                .to_string_lossy()
                .as_ref()
        )
    );
    assert_eq!(report.get("case_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(report.get("all_cases_proven").and_then(serde_json::Value::as_bool), Some(true));
    assert!(
        report
            .get("job_count")
            .and_then(serde_json::Value::as_u64)
            .is_some_and(|count| count > 100),
        "simulation report must stay aligned with the governed HPC job graph"
    );

    let cases = report.get("cases").and_then(serde_json::Value::as_array).expect("cases array");
    let relatedness_case = cases
        .iter()
        .find(|case_report| {
            case_report.get("case_id").and_then(serde_json::Value::as_str)
                == Some("relatedness_ibd_failure_isolates_demography_from_roh")
        })
        .expect("relatedness branch-isolation case");
    assert_eq!(
        relatedness_case.get("failed_job_id").and_then(serde_json::Value::as_str),
        Some("pipeline:relatedness-segments-vcf:vcf.ibd")
    );
    assert_eq!(
        relatedness_case.get("blocked_descendant_job_id").and_then(serde_json::Value::as_str),
        Some("pipeline:relatedness-segments-vcf:vcf.demography")
    );
    assert_eq!(
        relatedness_case.get("continued_sibling_job_id").and_then(serde_json::Value::as_str),
        Some("pipeline:relatedness-segments-vcf:vcf.roh")
    );
    assert_eq!(
        relatedness_case.get("blocked_only_descendants").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        relatedness_case.get("unrelated_branches_continue").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        relatedness_case.get("benchmark_jobs_continue").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(relatedness_case.get("proven").and_then(serde_json::Value::as_bool), Some(true));
    assert!(
        relatedness_case.get("blocked_job_ids").and_then(serde_json::Value::as_array).is_some_and(
            |job_ids| {
                job_ids.iter().any(|job_id| {
                    job_id.as_str() == Some("pipeline:relatedness-segments-vcf:vcf.demography")
                })
            }
        ),
        "the governed relatedness descendant must be blocked by the failed ibd node"
    );
    assert!(
        relatedness_case
            .get("continued_unrelated_job_ids")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|job_ids| {
                job_ids.iter().any(|job_id| {
                    job_id.as_str() == Some("pipeline:relatedness-segments-vcf:vcf.roh")
                })
            }),
        "the governed relatedness sibling branch must continue"
    );

    let relatedness_jobs = relatedness_case
        .get("jobs")
        .and_then(serde_json::Value::as_array)
        .expect("relatedness jobs");
    assert_eq!(
        job_by_id(relatedness_jobs, "pipeline:relatedness-segments-vcf:vcf.ibd")
            .get("status")
            .and_then(serde_json::Value::as_str),
        Some("failed")
    );
    assert_eq!(
        job_by_id(relatedness_jobs, "pipeline:relatedness-segments-vcf:vcf.demography")
            .get("status")
            .and_then(serde_json::Value::as_str),
        Some("blocked")
    );
    assert_eq!(
        job_by_id(relatedness_jobs, "pipeline:relatedness-segments-vcf:vcf.roh")
            .get("status")
            .and_then(serde_json::Value::as_str),
        Some("completed")
    );

    let reference_case = cases
        .iter()
        .find(|case_report| {
            case_report.get("case_id").and_then(serde_json::Value::as_str)
                == Some("reference_panel_prepare_failure_blocks_phasing_without_blocking_qc")
        })
        .expect("reference panel branch-isolation case");
    assert_eq!(
        reference_case.get("failed_job_id").and_then(serde_json::Value::as_str),
        Some("pipeline:reference-panel-imputation:vcf.prepare_reference_panel")
    );
    assert_eq!(
        reference_case.get("blocked_descendant_job_id").and_then(serde_json::Value::as_str),
        Some("pipeline:reference-panel-imputation:vcf.phasing")
    );
    assert_eq!(
        reference_case.get("continued_sibling_job_id").and_then(serde_json::Value::as_str),
        Some("pipeline:reference-panel-imputation:vcf.qc")
    );
    assert_eq!(
        reference_case.get("blocked_only_descendants").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        reference_case.get("unrelated_branches_continue").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        reference_case.get("benchmark_jobs_continue").and_then(serde_json::Value::as_bool),
        Some(true)
    );

    let reference_jobs =
        reference_case.get("jobs").and_then(serde_json::Value::as_array).expect("reference jobs");
    assert_eq!(
        job_by_id(
            reference_jobs,
            "pipeline:reference-panel-imputation:vcf.prepare_reference_panel"
        )
        .get("status")
        .and_then(serde_json::Value::as_str),
        Some("failed")
    );
    assert_eq!(
        job_by_id(reference_jobs, "pipeline:reference-panel-imputation:vcf.phasing")
            .get("status")
            .and_then(serde_json::Value::as_str),
        Some("blocked")
    );
    assert_eq!(
        job_by_id(reference_jobs, "pipeline:reference-panel-imputation:vcf.qc")
            .get("status")
            .and_then(serde_json::Value::as_str),
        Some("completed")
    );
}
