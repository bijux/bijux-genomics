#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_vcf_adapter_missing_input_tests_writes_governed_json_file() {
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
        .args(["bench", "readiness", "render-vcf-adapter-missing-input-tests"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let report_path = repo_root.join("target/bench-readiness/vcf-adapter-missing-input-tests.json");
    assert!(report_path.is_file(), "VCF missing-input report JSON must exist");

    let payload: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&report_path).expect("read report"))
            .expect("parse report json");
    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_adapter_missing_input_tests.v1")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(10));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert!(
        rows.iter().any(|row| {
            row.get("missing_input_role").and_then(serde_json::Value::as_str) == Some("fai")
                && row.get("artifact_id").and_then(serde_json::Value::as_str)
                    == Some("reference_fai")
                && row.get("passed").and_then(serde_json::Value::as_bool) == Some(true)
        }),
        "report file must retain the FASTA index probe row"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("missing_input_role").and_then(serde_json::Value::as_str)
                == Some("sample_metadata")
                && row.get("artifact_id").and_then(serde_json::Value::as_str)
                    == Some("sample_metadata_manifest")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("plink2")
        }),
        "report file must retain the sample metadata probe row"
    );
}
