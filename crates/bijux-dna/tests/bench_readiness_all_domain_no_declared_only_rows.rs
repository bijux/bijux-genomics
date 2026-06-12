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
fn bench_readiness_all_domain_no_declared_only_rows_reports_clean_active_scope() {
    let payload =
        run_cli_json(&["bench", "readiness", "render-all-domain-no-declared-only-rows", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.all_domain_no_declared_only_rows.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/all-domains/no-declared-only-rows.json")
    );
    assert_eq!(
        payload.get("lifecycle_active_row_count").and_then(serde_json::Value::as_u64),
        Some(138)
    );
    assert_eq!(
        payload.get("lifecycle_active_stage_count").and_then(serde_json::Value::as_u64),
        Some(67)
    );
    assert_eq!(
        payload.get("lifecycle_active_tool_count").and_then(serde_json::Value::as_u64),
        Some(71)
    );
    assert_eq!(payload.get("active_row_count").and_then(serde_json::Value::as_u64), Some(138));
    assert_eq!(payload.get("active_stage_count").and_then(serde_json::Value::as_u64), Some(67));
    assert_eq!(payload.get("active_tool_count").and_then(serde_json::Value::as_u64), Some(71));
    assert_eq!(payload.get("removed_row_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("removed_stage_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("removed_tool_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("violation_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let removed_adapter_status_counts = payload
        .get("removed_adapter_status_counts")
        .and_then(serde_json::Value::as_object)
        .expect("removed adapter status counts");
    assert!(
        removed_adapter_status_counts.is_empty(),
        "active scope must not remove any lifecycle-active rows for declaration-only adapters once executable-adapter filtering is enforced"
    );

    let removed_rows =
        payload.get("removed_rows").and_then(serde_json::Value::as_array).expect("removed rows");
    assert!(removed_rows.is_empty(), "active scope must not retain declared-only rows");

    let violations =
        payload.get("violations").and_then(serde_json::Value::as_array).expect("violations");
    assert!(violations.is_empty(), "active scope must not retain rows without executable adapters");
}
