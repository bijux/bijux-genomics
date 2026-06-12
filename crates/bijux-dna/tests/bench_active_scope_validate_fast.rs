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
fn bench_active_scope_validate_fast_reports_complete_fast_surface() {
    let payload = run_cli_json(&["bench", "active-scope", "validate", "--fast", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.active_scope_validate_fast.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("artifacts/bench-active-scope/validate-fast.json")
    );
    assert_eq!(payload.get("mode").and_then(serde_json::Value::as_str), Some("fast"));
    assert_eq!(
        payload.get("benchmark_root").and_then(serde_json::Value::as_str),
        Some("benchmarks")
    );
    assert_eq!(
        payload.get("schema_root").and_then(serde_json::Value::as_str),
        Some("benchmarks/schemas")
    );
    assert_eq!(
        payload.get("fixture_root").and_then(serde_json::Value::as_str),
        Some("benchmarks/tests/fixtures")
    );
    assert_eq!(payload.get("checked_surface_count").and_then(serde_json::Value::as_u64), Some(10));
    assert_eq!(payload.get("passed_surface_count").and_then(serde_json::Value::as_u64), Some(10));
    assert_eq!(payload.get("failed_surface_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let category_counts = payload
        .get("category_counts")
        .and_then(serde_json::Value::as_object)
        .expect("category counts");
    assert_eq!(category_counts.get("adapters").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(category_counts.get("commands").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(category_counts.get("configs").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(
        category_counts.get("expected_results").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(category_counts.get("fixtures").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(category_counts.get("outputs").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(category_counts.get("parsers").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(category_counts.get("reports").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(category_counts.get("schemas").and_then(serde_json::Value::as_u64), Some(1));

    let failed_category_counts = payload
        .get("failed_category_counts")
        .and_then(serde_json::Value::as_object)
        .expect("failed category counts");
    assert!(failed_category_counts.is_empty());

    let checks = payload.get("checks").and_then(serde_json::Value::as_array).expect("checks");
    assert_eq!(checks.len(), 10);
    assert!(checks
        .iter()
        .all(|check| { check.get("ok").and_then(serde_json::Value::as_bool) == Some(true) }));

    assert!(checks.iter().any(|check| {
        check.get("category").and_then(serde_json::Value::as_str) == Some("configs")
            && check.get("surface_id").and_then(serde_json::Value::as_str)
                == Some("all_domain_active_stage_tool_matrix")
            && check.get("output_path").and_then(serde_json::Value::as_str)
                == Some("benchmarks/readiness/all-domains/active-stage-tool-matrix.tsv")
            && check.get("detail").and_then(serde_json::Value::as_str)
                == Some("row_count=135, stage_count=65, tool_count=69")
    }));
    assert!(checks.iter().any(|check| {
        check.get("category").and_then(serde_json::Value::as_str) == Some("fixtures")
            && check.get("surface_id").and_then(serde_json::Value::as_str)
                == Some("benchmark_fixture_root_validation")
            && check.get("detail").and_then(serde_json::Value::as_str)
                == Some("checked_fixture_count=17, invalid_fixture_count=0")
    }));
    assert!(checks.iter().any(|check| {
        check.get("category").and_then(serde_json::Value::as_str) == Some("commands")
            && check.get("surface_id").and_then(serde_json::Value::as_str)
                == Some("all_domain_no_placeholder_command_check")
            && check.get("detail").and_then(serde_json::Value::as_str)
                == Some("valid_row_count=135, invalid_row_count=0, violation_count=0")
    }));
    assert!(checks.iter().any(|check| {
        check.get("category").and_then(serde_json::Value::as_str) == Some("reports")
            && check.get("surface_id").and_then(serde_json::Value::as_str)
                == Some("all_domain_report_map_coverage")
            && check.get("detail").and_then(serde_json::Value::as_str)
                == Some("covered_row_count=135, missing_row_count=0")
    }));
}
