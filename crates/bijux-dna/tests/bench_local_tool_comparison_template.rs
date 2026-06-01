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
fn bench_local_render_tool_comparison_template_json_reports_governed_51_row_slice() {
    let payload = run_cli_json(&["bench", "local", "render-tool-comparison-template", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_tool_comparison_template.v1")
    );
    assert_eq!(
        payload.get("tsv_output_path").and_then(serde_json::Value::as_str),
        Some("target/local-ready/tool-comparison-template.tsv")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(51));
    assert!(payload.get("rows").and_then(serde_json::Value::as_array).is_some_and(|rows| rows
        .len()
        == 51
        && rows.iter().all(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str).is_some()
                && row.get("tool_id").and_then(serde_json::Value::as_str).is_some()
                && row.get("runtime_seconds").and_then(serde_json::Value::as_str) == Some("1.0")
                && row
                    .get("memory_mb")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|memory| memory != "not_available")
                && row.get("output_metric").and_then(serde_json::Value::as_str)
                    == Some("not_available")
                && row.get("status").and_then(serde_json::Value::as_str) == Some("succeeded")
                && row.get("failure_reason").and_then(serde_json::Value::as_str)
                    == Some("not_applicable")
        })));
}
