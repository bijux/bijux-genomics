#![cfg(feature = "bam_downstream")]
#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

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

fn run_cli_json(args: &[&str]) -> serde_json::Value {
    let output = run_cli(args);
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
fn bench_readiness_adapter_missing_input_tests_report_structured_failures() {
    let payload =
        run_cli_json(&["bench", "readiness", "render-adapter-missing-input-tests", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.adapter_missing_input_tests.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/adapter-missing-input-tests.json")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(33));
    assert_eq!(payload.get("passed_row_count").and_then(serde_json::Value::as_u64), Some(33));
    assert_eq!(payload.get("failed_row_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(
        payload
            .get("missing_input_class_counts")
            .and_then(|value| value.get("database"))
            .and_then(serde_json::Value::as_u64),
        Some(4)
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 33);
    assert!(rows.iter().all(|row| {
        row.get("passed") == Some(&serde_json::Value::Bool(true))
            && row
                .get("observed_error")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|value| !value.trim().is_empty())
            && row
                .get("expected_error_fragment")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|value| !value.trim().is_empty())
    }));

    let taxonomy_row = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.screen_taxonomy")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("kaiju")
                && row.get("missing_input_role").and_then(serde_json::Value::as_str)
                    == Some("database_root")
        })
        .expect("taxonomy missing database root row");
    assert_eq!(
        taxonomy_row.get("missing_input_class").and_then(serde_json::Value::as_str),
        Some("database")
    );
    assert!(taxonomy_row
        .get("observed_error")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|value| value.contains("taxonomy database root is missing")));

    let recalibration_row = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.recalibration")
                && row.get("missing_input_role").and_then(serde_json::Value::as_str)
                    == Some("known_sites")
        })
        .expect("recalibration known-sites row");
    assert_eq!(
        recalibration_row.get("probe_kind").and_then(serde_json::Value::as_str),
        Some("local_smoke")
    );
    assert!(recalibration_row
        .get("observed_error")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|value| value.contains("known-sites fixture is missing")));
}
