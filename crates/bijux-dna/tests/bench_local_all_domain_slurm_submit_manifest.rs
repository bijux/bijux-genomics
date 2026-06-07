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
fn bench_local_render_all_domain_slurm_submit_manifest_reports_governed_job_slice() {
    let payload =
        run_cli_json(&["bench", "local", "render-all-domain-slurm-submit-manifest", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_all_domain_slurm_submit_manifest.v1")
    );
    assert_eq!(
        payload.get("root_path").and_then(serde_json::Value::as_str),
        Some("target/slurm-dry-run/all-domains")
    );
    assert_eq!(
        payload.get("manifest_path").and_then(serde_json::Value::as_str),
        Some("target/slurm-dry-run/all-domains/submit-manifest.json")
    );
    assert_eq!(
        payload.get("run_id").and_then(serde_json::Value::as_str),
        Some("all-domain-benchmark-dry-run")
    );
    assert_eq!(payload.get("job_count").and_then(serde_json::Value::as_u64), Some(213));
    assert_eq!(payload.get("benchmark_job_count").and_then(serde_json::Value::as_u64), Some(120));
    assert_eq!(
        payload.get("essential_pipeline_job_count").and_then(serde_json::Value::as_u64),
        Some(93)
    );

    let jobs = payload.get("jobs").and_then(serde_json::Value::as_array).expect("jobs array");
    assert_eq!(jobs.len(), 213);
    assert!(jobs.iter().all(|job| {
        job.get("job_id_local").and_then(serde_json::Value::as_str).is_some()
            && job.get("domain").and_then(serde_json::Value::as_str).is_some()
            && job.get("stage_id").and_then(serde_json::Value::as_str).is_some()
            && job.get("tool_id").and_then(serde_json::Value::as_str).is_some()
            && job.get("corpus_id").and_then(serde_json::Value::as_str).is_some()
            && job.get("asset_profile_id").and_then(serde_json::Value::as_str).is_some()
            && job.get("script_path").and_then(serde_json::Value::as_str).is_some_and(|path| {
                path.starts_with("target/slurm-dry-run/all-domains/") && path.ends_with(".sbatch")
            })
            && job.get("stdout").and_then(serde_json::Value::as_str).is_some()
            && job.get("stderr").and_then(serde_json::Value::as_str).is_some()
            && job
                .get("outputs")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|outputs| {
                    !outputs.is_empty()
                        && outputs.iter().all(|value| {
                            value.as_str().is_some_and(|path| {
                                path.starts_with("target/slurm-dry-run/")
                            })
                        })
                })
            && job.get("dependencies").and_then(serde_json::Value::as_array).is_some()
    }));
}
