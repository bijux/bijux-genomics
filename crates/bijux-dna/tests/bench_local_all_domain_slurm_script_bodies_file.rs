#![allow(clippy::expect_used)]

use std::fs;
use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_local_validate_all_domain_slurm_script_bodies_writes_owned_execution_commands() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(["bench", "local", "validate-all-domain-slurm-script-bodies"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let rendered_path = String::from_utf8(output.stdout).expect("stdout utf8");
    assert_eq!(
        rendered_path.trim(),
        "runs/bench/slurm-dry-run/all-domains/no-placeholder-report.json"
    );

    let report_path = repo_root.join(rendered_path.trim());
    assert!(report_path.is_file(), "all-domain slurm body report must exist");
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(&report_path).expect("read report"))
            .expect("parse report");
    assert_eq!(report.get("ok").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(report.get("script_count").and_then(serde_json::Value::as_u64), Some(213));

    let benchmark_script = repo_root.join(
        "runs/bench/slurm-dry-run/all-domains/benchmark-results/vcf/vcf_production_regression/vcf.stats/vcf_cohort/bcftools/job.sbatch",
    );
    assert!(benchmark_script.is_file(), "governed benchmark script must exist");
    let benchmark_body = fs::read_to_string(&benchmark_script).expect("read benchmark script");
    assert!(benchmark_body.contains("bijux-dna bench local execute-all-domain-benchmark-result"));
    assert!(benchmark_body
        .contains("--result-id vcf:vcf_production_regression:vcf.stats:vcf_cohort:bcftools"));
    assert!(!benchmark_body.contains("TODO"));
    assert!(!benchmark_body.contains("echo execute"));

    let pipeline_script =
        repo_root.join("runs/bench/slurm-dry-run/all-domains/essential-pipelines/relatedness-segments-vcf/vcf.ibd/job.sbatch");
    assert!(pipeline_script.is_file(), "governed pipeline script must exist");
    let pipeline_body = fs::read_to_string(&pipeline_script).expect("read pipeline script");
    assert!(pipeline_body.contains("bijux-dna bench local execute-essential-pipeline-node"));
    assert!(pipeline_body.contains("--pipeline-id relatedness-segments-vcf"));
    assert!(pipeline_body.contains("--node-id vcf.ibd"));
    assert!(!pipeline_body.contains("TODO"));
    assert!(!pipeline_body.contains("echo execute"));
}
