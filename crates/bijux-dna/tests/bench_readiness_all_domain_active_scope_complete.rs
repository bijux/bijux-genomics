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
fn bench_readiness_all_domain_active_scope_complete_reports_unambiguous_active_scope() {
    let payload =
        run_cli_json(&["bench", "readiness", "render-all-domain-active-scope-complete", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.all_domain_active_scope_complete.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/all-domains/ACTIVE_SCOPE_COMPLETE.json")
    );
    assert_eq!(payload.get("active_row_count").and_then(serde_json::Value::as_u64), Some(124));
    assert_eq!(payload.get("active_stage_count").and_then(serde_json::Value::as_u64), Some(57));
    assert_eq!(payload.get("active_tool_count").and_then(serde_json::Value::as_u64), Some(66));
    assert_eq!(payload.get("removed_row_count").and_then(serde_json::Value::as_u64), Some(21));
    assert_eq!(payload.get("checked_surface_count").and_then(serde_json::Value::as_u64), Some(19));
    assert_eq!(payload.get("passed_surface_count").and_then(serde_json::Value::as_u64), Some(19));
    assert_eq!(payload.get("failed_surface_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let checks = payload.get("checks").and_then(serde_json::Value::as_array).expect("checks");
    assert_eq!(checks.len(), 19);
    assert!(checks
        .iter()
        .all(|check| check.get("ok").and_then(serde_json::Value::as_bool) == Some(true)));

    assert!(checks.iter().any(|check| {
        check.get("surface_id").and_then(serde_json::Value::as_str)
            == Some("active_scope_validate_fast")
            && check.get("proof_paths").and_then(serde_json::Value::as_array).is_some_and(|paths| {
                paths.iter().any(|path| {
                    path.as_str() == Some("artifacts/bench-active-scope/validate-fast.json")
                })
            })
            && check.get("detail").and_then(serde_json::Value::as_str)
                == Some("checked_surface_count=10, failed_surface_count=0")
    }));
    assert!(checks.iter().any(|check| {
        check.get("surface_id").and_then(serde_json::Value::as_str)
            == Some("vcf_imputation_identity")
            && check.get("detail").and_then(serde_json::Value::as_str)
                == Some("legacy_imputation_row_count=0, impute_row_count=1, imputation_metrics_row_count=1")
    }));
    assert!(checks.iter().any(|check| {
        check.get("surface_id").and_then(serde_json::Value::as_str)
            == Some("vcf_postprocess_closure")
            && check.get("detail").and_then(serde_json::Value::as_str)
                == Some("active_postprocess_row_count=1, parser_fixture_match_count=1, output_contract_match_count=1, expected_result_match_count=1, report_map_match_count=1, local_job_match_count=1, command_audit_match_count=1")
    }));

    let failed_checks =
        payload.get("failed_checks").and_then(serde_json::Value::as_array).expect("failed checks");
    assert!(
        failed_checks.is_empty(),
        "final active-scope gate must keep explicit failing checks when the surface is ambiguous"
    );
}
