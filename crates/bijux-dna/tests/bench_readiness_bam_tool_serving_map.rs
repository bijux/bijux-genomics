#![allow(clippy::expect_used)]

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
fn bench_readiness_bam_tool_serving_map_reports_governed_bam_stage_rows() {
    let payload = run_cli_json(&["bench", "readiness", "render-bam-tool-serving-map", "--json"]);
    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.bam_tool_serving_map.v1")
    );
    assert_eq!(payload.get("domain").and_then(serde_json::Value::as_str), Some("bam"));
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/bench-readiness/bam-tool-serving-map.tsv")
    );
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(24));
    assert_eq!(payload.get("tool_count").and_then(serde_json::Value::as_u64), Some(23));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(
        payload.get("row_count").and_then(serde_json::Value::as_u64),
        Some(rows.len() as u64)
    );
    assert_eq!(rows.len(), 42, "BAM readiness map must retain the governed 42-row slice");
    assert!(
        rows.iter().any(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some("samtools")
                && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.validate")
                && row.get("support_status").and_then(serde_json::Value::as_str)
                    == Some("supported")
                && row.get("adapter_status").and_then(serde_json::Value::as_str)
                    == Some("plannable")
                && row.get("parser_status").and_then(serde_json::Value::as_str)
                    == Some("parser_fixture_validated")
                && row.get("corpus_status").and_then(serde_json::Value::as_str)
                    == Some("fixture:corpus-01-bam-mini")
        }),
        "BAM readiness map must retain the governed samtools validation row"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bcftools")
                && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.genotyping")
                && row.get("support_status").and_then(serde_json::Value::as_str)
                    == Some("missing_contract")
                && row.get("adapter_status").and_then(serde_json::Value::as_str)
                    == Some("declared_only")
                && row.get("corpus_status").and_then(serde_json::Value::as_str)
                    == Some("planner_only")
        }),
        "missing BAM tool contracts must remain visible instead of dropping benchmark rows"
    );
}
