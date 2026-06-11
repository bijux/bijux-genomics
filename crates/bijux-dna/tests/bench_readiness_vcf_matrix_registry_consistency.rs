#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli_json(args: &[&str]) -> serde_json::Value {
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
        .args(args)
        .output()
        .expect("run cli");

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
fn bench_readiness_vcf_matrix_registry_consistency_reports_governed_pass_state() {
    let payload =
        run_cli_json(&["bench", "readiness", "render-vcf-matrix-registry-consistency", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_matrix_registry_consistency.v1")
    );
    assert_eq!(payload.get("domain").and_then(serde_json::Value::as_str), Some("vcf"));
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/vcf-matrix-registry-consistency.json")
    );
    assert_eq!(payload.get("passes_gate"), Some(&serde_json::Value::Bool(true)));
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(20));
    assert_eq!(payload.get("matrix_row_count").and_then(serde_json::Value::as_u64), Some(23));
    assert_eq!(payload.get("registry_pair_count").and_then(serde_json::Value::as_u64), Some(44));
    assert_eq!(
        payload.get("benchmark_ready_registry_pair_count").and_then(serde_json::Value::as_u64),
        Some(15)
    );
    assert_eq!(
        payload.get("unregistered_matrix_pair_count").and_then(serde_json::Value::as_u64),
        Some(0)
    );
    assert_eq!(
        payload
            .get("missing_benchmark_ready_registry_pair_count")
            .and_then(serde_json::Value::as_u64),
        Some(0)
    );
    assert_eq!(payload.get("rows").and_then(serde_json::Value::as_array).map(Vec::len), Some(0));
}
