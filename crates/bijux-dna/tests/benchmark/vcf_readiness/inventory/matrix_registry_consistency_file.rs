#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

use crate::support;

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
fn bench_readiness_vcf_matrix_registry_consistency_file_writes_self_describing_report() {
    let output = run_cli(&["bench", "readiness", "render-vcf-matrix-registry-consistency"]);
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
        "artifacts/benchmark-smoke/readiness/vcf/vcf-matrix-registry-consistency.json"
    );

    let repo_root = support::repo_root().expect("repo root");
    let payload = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read VCF matrix/registry consistency report");
    let report: serde_json::Value = serde_json::from_str(&payload).expect("parse report JSON");

    assert_eq!(
        report.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_matrix_registry_consistency.v1")
    );
    assert_eq!(report.get("passes_gate"), Some(&serde_json::Value::Bool(true)));
    assert_eq!(report.get("stage_count").and_then(serde_json::Value::as_u64), Some(20));
    let matrix_row_count = report
        .get("matrix_row_count")
        .and_then(serde_json::Value::as_u64)
        .expect("matrix row count");
    let registry_pair_count = report
        .get("registry_pair_count")
        .and_then(serde_json::Value::as_u64)
        .expect("registry pair count");
    let benchmark_ready_registry_pair_count = report
        .get("benchmark_ready_registry_pair_count")
        .and_then(serde_json::Value::as_u64)
        .expect("benchmark-ready registry pair count");
    assert!(registry_pair_count >= matrix_row_count);
    assert!(matrix_row_count >= benchmark_ready_registry_pair_count);
    assert_eq!(
        report.get("unregistered_matrix_pair_count").and_then(serde_json::Value::as_u64),
        Some(0)
    );
    assert_eq!(
        report
            .get("missing_benchmark_ready_registry_pair_count")
            .and_then(serde_json::Value::as_u64),
        Some(0)
    );
    assert_eq!(report.get("rows").and_then(serde_json::Value::as_array).map(Vec::len), Some(0));
}
