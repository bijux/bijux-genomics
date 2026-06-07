#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_vcf_parser_failure_tests_writes_governed_json_file() {
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
        .args(["bench", "readiness", "render-vcf-parser-failure-tests"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let report_path = repo_root.join("benchmarks/readiness/vcf-parser-failure-tests.json");
    assert!(report_path.is_file(), "VCF parser failure report JSON must exist");

    let payload: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&report_path).expect("read report"))
            .expect("parse report json");
    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_parser_failure_tests.v1")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(7));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert!(rows.iter().any(|row| {
        row.get("failure_reason").and_then(serde_json::Value::as_str) == Some("malformed_pca_table")
            && row.get("probe_artifact_path").and_then(serde_json::Value::as_str)
                == Some(
                    "artifacts/bench-readiness/vcf-parser-failure-tests/malformed-pca-table/fixture/raw.evec",
                )
            && row.get("expected_error_fragment").and_then(serde_json::Value::as_str)
                == Some("does not contain any numeric components")
    }));
    assert!(rows.iter().any(|row| {
        row.get("failure_reason").and_then(serde_json::Value::as_str) == Some("empty_output")
            && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.call")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bcftools")
    }));
}
