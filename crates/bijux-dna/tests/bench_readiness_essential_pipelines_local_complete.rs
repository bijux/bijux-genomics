#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli_json() -> serde_json::Value {
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
        .args(["bench", "readiness", "render-essential-pipelines-local-complete", "--json"])
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
fn bench_readiness_essential_pipelines_local_complete_reports_governed_pass_state() {
    let payload = run_cli_json();

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.essential_pipelines_local_complete.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/pipelines/ESSENTIAL_PIPELINES_LOCAL_COMPLETE.json")
    );
    assert_eq!(payload.get("pipeline_count").and_then(serde_json::Value::as_u64), Some(10));
    assert_eq!(payload.get("node_count").and_then(serde_json::Value::as_u64), Some(93));
    assert_eq!(payload.get("completed_node_count").and_then(serde_json::Value::as_u64), Some(93));
    assert_eq!(payload.get("failing_node_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("rendered_node_count").and_then(serde_json::Value::as_u64), Some(93));
    assert_eq!(
        payload.get("structured_skip_node_count").and_then(serde_json::Value::as_u64),
        Some(0)
    );
    assert_eq!(payload.get("corpus_asset_row_count").and_then(serde_json::Value::as_u64), Some(93));
    assert_eq!(
        payload.get("rendered_command_row_count").and_then(serde_json::Value::as_u64),
        Some(93)
    );
    assert_eq!(payload.get("report_map_row_count").and_then(serde_json::Value::as_u64), Some(267));
    assert_eq!(payload.get("declared_output_count").and_then(serde_json::Value::as_u64), Some(267));
    assert_eq!(payload.get("reported_output_count").and_then(serde_json::Value::as_u64), Some(267));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 93);
    assert!(rows
        .iter()
        .all(|row| row.get("ok").and_then(serde_json::Value::as_bool) == Some(true)));
    assert!(rows.iter().any(|row| {
        row.get("pipeline_id").and_then(serde_json::Value::as_str)
            == Some("bam-genotyping-to-vcf-downstream")
            && row.get("node_id").and_then(serde_json::Value::as_str) == Some("bam.genotyping")
            && row.get("command_source").and_then(serde_json::Value::as_str)
                == Some("local_stage_materialization")
    }));
}
