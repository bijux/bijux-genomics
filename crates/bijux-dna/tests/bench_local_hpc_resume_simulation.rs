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
    let report_path = temp_dir.path().join("resume-simulation.json");
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
fn bench_local_render_hpc_resume_simulation_proves_skip_and_rerun_rules() {
    let repo_root = support::repo_root().expect("repo root");
    let (_temp_dir, report_path) = render_path(&repo_root, "render-hpc-resume-simulation-");
    let report_arg = report_path.to_string_lossy().into_owned();

    let output =
        run_cli(&["bench", "local", "render-hpc-resume-simulation", "--output", &report_arg]);
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
        Some("bijux.bench.local_hpc_resume_simulation.v1")
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
    assert!(
        report
            .get("job_count")
            .and_then(serde_json::Value::as_u64)
            .is_some_and(|count| count > 100),
        "resume simulation must stay aligned with the governed HPC job graph"
    );
    assert_eq!(
        report.get("failed_stage_result_job_count").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(
        report.get("missing_stage_result_job_count").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(
        report.get("stale_partial_output_job_count").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(
        report.get("invalid_stage_result_job_count").and_then(serde_json::Value::as_u64),
        Some(0)
    );
    assert_eq!(
        report.get("dependency_rerun_job_count").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(report.get("rerun_job_count").and_then(serde_json::Value::as_u64), Some(4));
    assert_eq!(
        report
            .get("behavior")
            .and_then(|value| value.get("proven"))
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );

    let jobs = report.get("jobs").and_then(serde_json::Value::as_array).expect("jobs array");
    assert_eq!(
        job_by_id(jobs, "pipeline:relatedness-segments-vcf:vcf.ibd")
            .get("completion_state")
            .and_then(serde_json::Value::as_str),
        Some("failed_stage_result_manifest")
    );
    assert_eq!(
        job_by_id(jobs, "pipeline:relatedness-segments-vcf:vcf.ibd")
            .get("resume_action")
            .and_then(serde_json::Value::as_str),
        Some("rerun")
    );
    assert_eq!(
        job_by_id(jobs, "pipeline:relatedness-segments-vcf:vcf.demography")
            .get("resume_action")
            .and_then(serde_json::Value::as_str),
        Some("rerun")
    );
    assert_eq!(
        job_by_id(jobs, "pipeline:relatedness-segments-vcf:vcf.demography")
            .get("reason")
            .and_then(serde_json::Value::as_str),
        Some("upstream_dependency_rerun")
    );
    assert_eq!(
        job_by_id(jobs, "pipeline:relatedness-segments-vcf:vcf.roh")
            .get("resume_action")
            .and_then(serde_json::Value::as_str),
        Some("skip")
    );
    assert_eq!(
        job_by_id(jobs, "benchmark:vcf:vcf_production_regression:vcf.stats:vcf_cohort:bcftools")
            .get("completion_state")
            .and_then(serde_json::Value::as_str),
        Some("missing_stage_result_manifest")
    );
    assert_eq!(
        job_by_id(
            jobs,
            "benchmark:bam:corpus-01-adna-bam-mini:bam.contamination:sample-set:contammix"
        )
        .get("completion_state")
        .and_then(serde_json::Value::as_str),
        Some("stale_partial_outputs")
    );
    assert_eq!(
        job_by_id(jobs, "benchmark:vcf:vcf_production_regression:vcf.qc:vcf_cohort:bcftools")
            .get("resume_action")
            .and_then(serde_json::Value::as_str),
        Some("skip")
    );
}
