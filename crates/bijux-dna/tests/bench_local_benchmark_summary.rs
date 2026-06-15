#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli_json(args: &[&str]) -> serde_json::Value {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    let output = Command::new("cargo")
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(["run", "-q", "-p", "bijux-dna", "--features", "bam_downstream", "--"])
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

#[cfg(feature = "bam_downstream")]
#[test]
fn bench_local_render_benchmark_summary_json_reports_governed_51_stage_slice() {
    let payload = run_cli_json(&["bench", "local", "render-benchmark-summary", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_benchmark_summary.v1")
    );
    assert_eq!(
        payload.get("report_output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/local-ready/benchmark-summary.json")
    );
    assert_eq!(
        payload.get("markdown_output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/local-ready/benchmark-summary.md")
    );
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(51));
    assert_eq!(payload.get("ready_stage_count").and_then(serde_json::Value::as_u64), Some(51));
    assert_eq!(payload.get("incomplete_stage_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("failed_stage_count").and_then(serde_json::Value::as_u64), Some(0));
    assert!(payload.get("stages").and_then(serde_json::Value::as_array).is_some_and(|stages| {
        stages.len() == 51
            && stages.iter().all(|stage| {
                stage.get("stage_id").and_then(serde_json::Value::as_str).is_some()
                    && stage.get("tool_id").and_then(serde_json::Value::as_str).is_some()
                    && stage.get("readiness_status").and_then(serde_json::Value::as_str)
                        == Some("ready")
                    && stage.get("runtime_status").and_then(serde_json::Value::as_str)
                        == Some("succeeded")
                    && stage.get("missing_output_count").and_then(serde_json::Value::as_u64)
                        == Some(0)
            })
    }));
}
