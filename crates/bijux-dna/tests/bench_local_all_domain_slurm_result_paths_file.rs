#![allow(clippy::expect_used)]

use std::fs;
use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_local_validate_all_domain_slurm_result_paths_writes_governed_report_path() {
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
        .args(["bench", "local", "validate-all-domain-slurm-result-paths"])
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
    assert_eq!(rendered_path.trim(), "target/slurm-dry-run/all-domains/path-convention-check.json");

    let report_path = repo_root.join(rendered_path.trim());
    assert!(report_path.is_file(), "all-domain path convention report must exist");
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(&report_path).expect("read report"))
            .expect("parse report");
    assert_eq!(report.get("ok").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(report.get("job_count").and_then(serde_json::Value::as_u64), Some(213));

    let jobs = report.get("jobs").and_then(serde_json::Value::as_array).expect("jobs array");
    let benchmark_job = jobs
        .iter()
        .find(|job| {
            job.get("job_id_local").and_then(serde_json::Value::as_str)
                == Some("benchmark:vcf:vcf_production_regression:vcf.stats:vcf_cohort:bcftools")
        })
        .expect("benchmark job");
    assert_eq!(
        benchmark_job.get("checked_path_count").and_then(serde_json::Value::as_u64),
        Some(5)
    );

    let manifest_path = repo_root.join("target/slurm-dry-run/all-domains/submit-manifest.json");
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(&manifest_path).expect("read manifest"))
            .expect("parse manifest");
    let pipeline_manifest_job = manifest
        .get("jobs")
        .and_then(serde_json::Value::as_array)
        .and_then(|jobs| {
            jobs.iter().find(|job| {
                job.get("job_id_local").and_then(serde_json::Value::as_str)
                    == Some("pipeline:relatedness-segments-vcf:vcf.demography")
            })
        })
        .expect("pipeline manifest job");
    assert_eq!(
        pipeline_manifest_job.get("stdout").and_then(serde_json::Value::as_str),
        Some(
            "target/slurm-dry-run/all-domains/runs/all-domain-benchmark-dry-run/vcf/relatedness-segments-vcf/vcf.demography/ibdne/vcf_production_regression/sample-set/stdout.log"
        )
    );
}
