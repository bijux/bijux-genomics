#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_fastq_validate_reads_ready_writes_governed_json_file() {
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
        .args(["bench", "readiness", "render-fastq-validate-reads-ready"])
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
    assert_eq!(rendered_path.trim(), "benchmarks/readiness/fastq/validate-reads-ready.json");

    let payload = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read FASTQ validate-reads readiness report");
    let report: serde_json::Value = serde_json::from_str(&payload).expect("parse report");

    assert_eq!(
        report.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.fastq_validate_reads_ready.v1")
    );
    assert_eq!(report.get("active_row_count").and_then(serde_json::Value::as_u64), Some(5));
    assert_eq!(report.get("complete_row_count").and_then(serde_json::Value::as_u64), Some(5));
    assert_eq!(report.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let row = report
        .get("rows")
        .and_then(serde_json::Value::as_array)
        .and_then(|rows| rows.first())
        .expect("first row");
    assert_eq!(
        row.get("stage_id").and_then(serde_json::Value::as_str),
        Some("fastq.validate_reads")
    );
    assert!(row.get("expected_outputs").and_then(serde_json::Value::as_array).is_some_and(
        |outputs| {
            outputs.iter().any(|value| value.as_str() == Some("validation_report"))
                && outputs.iter().any(|value| value.as_str() == Some("validated_reads_manifest"))
        }
    ));
    assert!(row.get("raw_output_artifact_ids").and_then(serde_json::Value::as_array).is_some_and(
        |outputs| {
            outputs.iter().any(|value| value.as_str() == Some("validated_reads_manifest"))
        }
    ));
    assert!(row.get("smoke_read_count_totals").and_then(serde_json::Value::as_array).is_some_and(
        |totals| {
            totals
                == &[
                    serde_json::Value::Number(2_u64.into()),
                    serde_json::Value::Number(4_u64.into()),
                ]
        }
    ));
    assert!(row
        .get("smoke_failure_classes")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|classes| { classes == &[serde_json::Value::String("none".to_string())] }));
    assert_eq!(
        row.get("reason").and_then(serde_json::Value::as_str),
        Some(
            "retained FASTQ validator `fastq_scan` keeps active scope, command, output, parser, expected-result, report, and normalized validation proof for `fastq.validate_reads`",
        )
    );
}
