#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::collections::BTreeSet;
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
fn bench_readiness_all_domain_failure_classification_reports_each_required_class() {
    let payload =
        run_cli_json(&["bench", "readiness", "render-all-domain-failure-classification", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.all_domain_failure_classification.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/failure-classification-all-domains.json")
    );
    assert_eq!(
        payload.get("fixture_root").and_then(serde_json::Value::as_str),
        Some("runs/bench/readiness-probes/all-domains/failure-classification")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(7));
    assert_eq!(payload.get("triggered_row_count").and_then(serde_json::Value::as_u64), Some(7));
    assert_eq!(payload.get("required_class_count").and_then(serde_json::Value::as_u64), Some(7));
    assert_eq!(payload.get("triggered_class_count").and_then(serde_json::Value::as_u64), Some(7));
    assert_eq!(payload.get("missing_class_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("passes_behavior_test"), Some(&serde_json::Value::Bool(true)));

    let missing_class_ids = payload
        .get("missing_class_ids")
        .and_then(serde_json::Value::as_array)
        .expect("missing class ids");
    assert!(missing_class_ids.is_empty());

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 7);
    assert!(rows.iter().all(|row| row.get("triggered") == Some(&serde_json::Value::Bool(true))));
    assert_eq!(
        rows.iter()
            .map(|row| row.get("class_id").and_then(serde_json::Value::as_str).expect("class id"))
            .collect::<BTreeSet<_>>(),
        [
            "missing_input",
            "tool_not_found",
            "command_failed",
            "missing_output",
            "parser_failed",
            "insufficient_data",
            "unsupported_pair",
        ]
        .into_iter()
        .collect()
    );

    let tool_not_found = rows
        .iter()
        .find(|row| {
            row.get("class_id").and_then(serde_json::Value::as_str) == Some("tool_not_found")
        })
        .expect("tool not found row");
    assert_eq!(
        tool_not_found.get("observed_status").and_then(serde_json::Value::as_str),
        Some("tool_not_found")
    );
    assert!(tool_not_found.get("result_id").is_some_and(|value| value.as_str().is_some()));

    let unsupported_pair = rows
        .iter()
        .find(|row| {
            row.get("class_id").and_then(serde_json::Value::as_str) == Some("unsupported_pair")
        })
        .expect("unsupported pair row");
    assert_eq!(unsupported_pair.get("domain").and_then(serde_json::Value::as_str), Some("vcf"));
    assert_eq!(
        unsupported_pair.get("stage_id").and_then(serde_json::Value::as_str),
        Some("vcf.filter")
    );
    assert_eq!(
        unsupported_pair.get("tool_id").and_then(serde_json::Value::as_str),
        Some("samtools")
    );

    let insufficient_data = rows
        .iter()
        .find(|row| {
            row.get("class_id").and_then(serde_json::Value::as_str) == Some("insufficient_data")
        })
        .expect("insufficient data row");
    assert_eq!(
        insufficient_data.get("observed_status").and_then(serde_json::Value::as_str),
        Some("insufficient_data")
    );
}
