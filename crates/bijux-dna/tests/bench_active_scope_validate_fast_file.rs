#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_active_scope_validate_fast_writes_disposable_report_file() {
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
        .args(["bench", "active-scope", "validate", "--fast"])
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
    assert_eq!(rendered_path.trim(), "artifacts/bench-active-scope/validate-fast.json");

    let payload = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read active-scope fast validation report");
    let payload: serde_json::Value = serde_json::from_str(&payload).expect("parse report json");

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.active_scope_validate_fast.v1")
    );
    assert_eq!(payload.get("checked_surface_count").and_then(serde_json::Value::as_u64), Some(10));
    assert_eq!(payload.get("passed_surface_count").and_then(serde_json::Value::as_u64), Some(10));
    assert_eq!(payload.get("failed_surface_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let checks = payload.get("checks").and_then(serde_json::Value::as_array).expect("checks");
    assert_eq!(checks.len(), 10);
    assert!(checks.iter().any(|check| {
        check.get("surface_id").and_then(serde_json::Value::as_str)
            == Some("all_domain_schema_validation")
            && check.get("output_path").and_then(serde_json::Value::as_str)
                == Some("benchmarks/readiness/all-domain-schema-validation.json")
    }));
    assert!(checks.iter().any(|check| {
        check.get("surface_id").and_then(serde_json::Value::as_str)
            == Some("all_domain_local_job_coverage")
            && check.get("output_path").and_then(serde_json::Value::as_str)
                == Some("benchmarks/readiness/all-domains/local-job-coverage.tsv")
    }));
}
