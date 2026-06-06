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
fn bench_readiness_essential_pipeline_partial_resume_tracks_resume_actions() {
    let payload =
        run_cli_json(&["bench", "readiness", "render-essential-pipeline-partial-resume", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.essential_pipeline_partial_resume.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/bench-readiness/essential-pipeline-partial-resume.json")
    );
    assert_eq!(
        payload.get("simulation_root").and_then(serde_json::Value::as_str),
        Some("target/bench-readiness/essential-pipeline-partial-resume-tree")
    );
    assert_eq!(payload.get("pipeline_count").and_then(serde_json::Value::as_u64), Some(10));
    assert_eq!(payload.get("node_count").and_then(serde_json::Value::as_u64), Some(93));
    assert_eq!(
        payload.get("valid_completed_node_count").and_then(serde_json::Value::as_u64),
        Some(92)
    );
    assert_eq!(
        payload.get("invalid_manifest_node_count").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(
        payload.get("missing_manifest_node_count").and_then(serde_json::Value::as_u64),
        Some(0)
    );
    assert_eq!(payload.get("skip_node_count").and_then(serde_json::Value::as_u64), Some(91));
    assert_eq!(payload.get("rerun_node_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(
        payload.get("seeded_invalid_node_id").and_then(serde_json::Value::as_str),
        Some("relatedness-segments-vcf::vcf.ibd")
    );
    assert_eq!(payload.get("passes_behavior_test"), Some(&serde_json::Value::Bool(true)));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 93);

    let invalid_row = rows
        .iter()
        .find(|row| {
            row.get("pipeline_id").and_then(serde_json::Value::as_str)
                == Some("relatedness-segments-vcf")
                && row.get("node_id").and_then(serde_json::Value::as_str) == Some("vcf.ibd")
        })
        .expect("seeded invalid row");
    assert_eq!(
        invalid_row.get("completion_state").and_then(serde_json::Value::as_str),
        Some("invalid_stage_result_manifest")
    );
    assert_eq!(invalid_row.get("resume_action").and_then(serde_json::Value::as_str), Some("rerun"));
    assert_eq!(
        invalid_row.get("reason").and_then(serde_json::Value::as_str),
        Some("invalid_stage_result_manifest")
    );

    let downstream_row = rows
        .iter()
        .find(|row| {
            row.get("pipeline_id").and_then(serde_json::Value::as_str)
                == Some("relatedness-segments-vcf")
                && row.get("node_id").and_then(serde_json::Value::as_str) == Some("vcf.demography")
        })
        .expect("downstream rerun row");
    assert_eq!(
        downstream_row.get("completion_state").and_then(serde_json::Value::as_str),
        Some("valid_completed")
    );
    assert_eq!(
        downstream_row.get("resume_action").and_then(serde_json::Value::as_str),
        Some("rerun")
    );
    assert_eq!(
        downstream_row.get("reason").and_then(serde_json::Value::as_str),
        Some("upstream_dependency_rerun")
    );

    let unrelated_row = rows
        .iter()
        .find(|row| {
            row.get("pipeline_id").and_then(serde_json::Value::as_str)
                == Some("relatedness-segments-vcf")
                && row.get("node_id").and_then(serde_json::Value::as_str) == Some("vcf.roh")
        })
        .expect("unrelated continued row");
    assert_eq!(
        unrelated_row.get("completion_state").and_then(serde_json::Value::as_str),
        Some("valid_completed")
    );
    assert_eq!(
        unrelated_row.get("resume_action").and_then(serde_json::Value::as_str),
        Some("skip")
    );
    assert_eq!(
        unrelated_row.get("unrelated_branch_continues"),
        Some(&serde_json::Value::Bool(true))
    );
}
