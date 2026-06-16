#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::fs;
use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_bam_insert_size_gc_bias_ready_writes_governed_json_file() {
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
        .args(["bench", "readiness", "render-bam-insert-size-gc-bias-ready"])
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
    assert_eq!(rendered_path.trim(), "benchmarks/readiness/bam/insert-size-gc-bias-ready.json");

    let payload = fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read BAM insert-size/gc-bias readiness JSON");
    let report: serde_json::Value =
        serde_json::from_str(&payload).expect("parse BAM insert-size/gc-bias readiness JSON");

    assert_eq!(
        report.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.bam_insert_size_gc_bias_ready.v1")
    );
    assert_eq!(report.get("complete_row_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(report.get("violation_count").and_then(serde_json::Value::as_u64), Some(0));

    let required_output_ids = report
        .get("required_output_ids_by_stage")
        .and_then(serde_json::Value::as_object)
        .expect("required output ids by stage");
    assert!(required_output_ids
        .get("bam.insert_size")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|outputs| {
            outputs.iter().any(|output| output.as_str() == Some("insert_size_histogram"))
                && outputs.iter().any(|output| output.as_str() == Some("summary"))
        }));
    assert!(required_output_ids
        .get("bam.gc_bias")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|outputs| {
            outputs.iter().any(|output| output.as_str() == Some("gc_bias_report"))
                && outputs.iter().any(|output| output.as_str() == Some("gc_bias_plot"))
        }));
}
