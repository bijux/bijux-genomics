#![allow(clippy::expect_used, clippy::too_many_lines)]

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

#[test]
fn bench_local_vcf_reference_compatibility_writes_governed_json_file() {
    let output = run_cli(&["bench", "local", "validate-vcf-reference-compatibility"]);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "benchmarks/readiness/local-ready/vcf/reference-compatibility.json"
    );

    let repo_root = support::repo_root().expect("repo root");
    let report_path =
        repo_root.join("benchmarks/readiness/local-ready/vcf/reference-compatibility.json");
    let raw = std::fs::read_to_string(&report_path).expect("read report");
    let payload: serde_json::Value = serde_json::from_str(&raw).expect("parse report json");

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_vcf_reference_compatibility.v1")
    );
    assert_eq!(payload.get("status").and_then(serde_json::Value::as_str), Some("compatible"));
    assert_eq!(
        payload
            .get("reference_contigs")
            .and_then(serde_json::Value::as_array)
            .map(|values| values.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>()),
        Some(vec!["chr1", "chr2"])
    );
    assert_eq!(
        payload
            .get("vcf_contigs")
            .and_then(serde_json::Value::as_array)
            .map(|values| values.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>()),
        Some(vec!["chr1", "chr2"])
    );
}
