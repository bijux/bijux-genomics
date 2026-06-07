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
fn bench_local_render_all_domain_slurm_scripts_reports_governed_job_counts() {
    let payload = run_cli_json(&["bench", "local", "render-all-domain-slurm-scripts", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_all_domain_slurm_scripts.v1")
    );
    assert_eq!(
        payload.get("output_root").and_then(serde_json::Value::as_str),
        Some("target/slurm-dry-run/all-domains")
    );
    assert_eq!(payload.get("script_count").and_then(serde_json::Value::as_u64), Some(213));
    assert_eq!(payload.get("benchmark_job_count").and_then(serde_json::Value::as_u64), Some(120));
    assert_eq!(
        payload.get("essential_pipeline_job_count").and_then(serde_json::Value::as_u64),
        Some(93)
    );
    assert_eq!(payload.get("pipeline_count").and_then(serde_json::Value::as_u64), Some(10));

    let domain_counts =
        payload.get("domain_counts").and_then(serde_json::Value::as_object).expect("domain counts");
    assert_eq!(domain_counts.get("fastq").and_then(serde_json::Value::as_u64), Some(92));
    assert_eq!(domain_counts.get("bam").and_then(serde_json::Value::as_u64), Some(80));
    assert_eq!(domain_counts.get("vcf").and_then(serde_json::Value::as_u64), Some(41));

    let benchmark_domain_counts = payload
        .get("benchmark_domain_counts")
        .and_then(serde_json::Value::as_object)
        .expect("benchmark domain counts");
    assert_eq!(benchmark_domain_counts.get("fastq").and_then(serde_json::Value::as_u64), Some(63));
    assert_eq!(benchmark_domain_counts.get("bam").and_then(serde_json::Value::as_u64), Some(49));
    assert_eq!(benchmark_domain_counts.get("vcf").and_then(serde_json::Value::as_u64), Some(8));

    let scripts =
        payload.get("scripts").and_then(serde_json::Value::as_array).expect("scripts array");
    assert_eq!(scripts.len(), 213);
    assert!(scripts.iter().any(|entry| {
        entry.get("job_kind").and_then(serde_json::Value::as_str) == Some("benchmark_result")
            && entry.get("domain").and_then(serde_json::Value::as_str) == Some("vcf")
            && entry.get("script_path").and_then(serde_json::Value::as_str).is_some_and(|path| {
                path.starts_with("target/slurm-dry-run/all-domains/benchmark-results/vcf/")
                    && path.ends_with("/job.sbatch")
            })
    }));
    assert!(scripts.iter().any(|entry| {
        entry.get("job_kind").and_then(serde_json::Value::as_str)
            == Some("essential_pipeline_node")
            && entry.get("pipeline_id").and_then(serde_json::Value::as_str)
                == Some("relatedness-segments-vcf")
            && entry.get("node_id").and_then(serde_json::Value::as_str) == Some("vcf.ibd")
            && entry
                .get("script_path")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|path| {
                    path == "target/slurm-dry-run/all-domains/essential-pipelines/relatedness-segments-vcf/vcf.ibd/job.sbatch"
                })
    }));
}
