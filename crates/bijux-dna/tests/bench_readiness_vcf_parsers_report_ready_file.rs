#![allow(clippy::expect_used)]

use std::fs;
use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_vcf_parsers_report_ready_writes_governed_json_file() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");
    let output_path = repo_root.join("target/bench-readiness/VCF_PARSERS_REPORT_READY.json");

    if output_path.exists() {
        fs::remove_file(&output_path).expect("remove stale output");
    }

    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(["bench", "readiness", "render-vcf-parsers-report-ready"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout utf8").trim(),
        "target/bench-readiness/VCF_PARSERS_REPORT_READY.json"
    );

    let payload = fs::read_to_string(&output_path).expect("read output file");
    let json: serde_json::Value = serde_json::from_str(&payload).expect("parse output json");
    assert_eq!(
        json.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_parsers_report_ready.v1")
    );
    assert_eq!(json.get("ok"), Some(&serde_json::Value::Bool(true)));
    assert_eq!(json.get("checked_goal_count").and_then(serde_json::Value::as_u64), Some(15));
}
