#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

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

fn run_cli_json(args: &[&str]) -> serde_json::Value {
    let output = run_cli(args);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("parse stdout as json")
}

#[test]
fn bench_local_validate_all_domain_slurm_result_paths_reports_governed_run_root() {
    let payload =
        run_cli_json(&["bench", "local", "validate-all-domain-slurm-result-paths", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_all_domain_slurm_path_convention.v1")
    );
    assert_eq!(
        payload.get("root_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/slurm-dry-run/all-domains")
    );
    assert_eq!(
        payload.get("manifest_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/slurm-dry-run/all-domains/submit-manifest.json")
    );
    assert_eq!(
        payload.get("report_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/slurm-dry-run/all-domains/path-convention-check.json")
    );
    assert_eq!(payload.get("job_count").and_then(serde_json::Value::as_u64), Some(214));
    assert_eq!(payload.get("finding_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let jobs = payload.get("jobs").and_then(serde_json::Value::as_array).expect("jobs array");
    assert_eq!(jobs.len(), 214);
    assert!(jobs.iter().all(|job| {
        job.get("ok").and_then(serde_json::Value::as_bool) == Some(true)
            && job
                .get("checked_path_count")
                .and_then(serde_json::Value::as_u64)
                .is_some_and(|count| count >= 3)
            && job
                .get("findings")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|findings| findings.is_empty())
    }));

    let benchmark_job = jobs
        .iter()
        .find(|job| {
            job.get("job_id_local").and_then(serde_json::Value::as_str)
                == Some("benchmark:vcf:vcf_production_regression:vcf.stats:vcf_cohort:bcftools")
        })
        .expect("benchmark job");
    assert_eq!(benchmark_job.get("sample_scope"), Some(&serde_json::Value::Null));

    let pipeline_job = jobs
        .iter()
        .find(|job| {
            job.get("job_id_local").and_then(serde_json::Value::as_str)
                == Some("pipeline:relatedness-segments-vcf:vcf.demography")
        })
        .expect("pipeline job");
    assert_eq!(
        pipeline_job.get("sample_scope").and_then(serde_json::Value::as_str),
        Some("sample-set")
    );
}
