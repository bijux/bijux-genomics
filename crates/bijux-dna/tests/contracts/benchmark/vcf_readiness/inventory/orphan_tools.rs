#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

use crate::support;

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
fn bench_readiness_vcf_orphan_tools_reports_governed_decisions() {
    let payload = run_cli_json(&["bench", "readiness", "render-vcf-orphan-tools", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_orphan_tools.v1")
    );
    assert_eq!(payload.get("domain").and_then(serde_json::Value::as_str), Some("vcf"));
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("artifacts/benchmark-smoke/readiness/vcf/vcf-orphan-tools.tsv")
    );
    let orphan_count =
        payload.get("orphan_count").and_then(serde_json::Value::as_u64).expect("orphan count");
    let required_tool_count = payload
        .get("required_tool_count")
        .and_then(serde_json::Value::as_u64)
        .expect("required tool count");
    let served_tool_count = payload
        .get("served_tool_count")
        .and_then(serde_json::Value::as_u64)
        .expect("served tool count");
    assert_eq!(
        payload.get("registered_tool_count").and_then(serde_json::Value::as_u64),
        Some(required_tool_count)
    );
    assert_eq!(served_tool_count, required_tool_count);
    assert_eq!(orphan_count, 0);

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len() as u64, orphan_count);
    assert!(rows.is_empty());
}
