#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_vcf_missing_result_report_writes_governed_json_file() {
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
        .args(["bench", "readiness", "render-vcf-missing-result-report"])
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
    assert_eq!(rendered_path.trim(), "benchmarks/readiness/vcf-missing-result-report-test.json");

    let payload = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read VCF missing-result report");
    let parsed: serde_json::Value = serde_json::from_str(&payload).expect("parse report json");

    assert_eq!(
        parsed.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_missing_result_report.v1")
    );
    assert_eq!(parsed.get("missing_result_row_count").and_then(serde_json::Value::as_u64), Some(1));
    assert!(
        parsed.get("rows").and_then(serde_json::Value::as_array).is_some_and(|rows| {
            rows.iter().any(|row| {
                row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.stats")
                    && row.get("result_status").and_then(serde_json::Value::as_str)
                        == Some("missing_result")
            })
        }),
        "JSON file must retain the governed missing VCF stats row"
    );
}
