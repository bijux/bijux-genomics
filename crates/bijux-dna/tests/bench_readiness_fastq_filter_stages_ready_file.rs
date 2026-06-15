#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_fastq_filter_stages_ready_writes_governed_json_file() {
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
        .args(["bench", "readiness", "render-fastq-filter-stages-ready"])
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
    assert_eq!(rendered_path.trim(), "benchmarks/readiness/fastq/filter-stages-ready.json");

    let payload = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read FASTQ filter stages readiness report");
    let report: serde_json::Value = serde_json::from_str(&payload).expect("parse report");

    assert_eq!(
        report.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.fastq_filter_stages_ready.v1")
    );
    assert_eq!(report.get("active_row_count").and_then(serde_json::Value::as_u64), Some(6));
    assert_eq!(report.get("complete_row_count").and_then(serde_json::Value::as_u64), Some(6));
    assert_eq!(report.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let row = report
        .get("rows")
        .and_then(serde_json::Value::as_array)
        .and_then(|rows| {
            rows.iter().find(|row| {
                row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("fastq.filter_reads")
                    && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("fastp")
            })
        })
        .expect("filter_reads fastp row");
    assert_eq!(
        row.get("command_readiness_kind").and_then(serde_json::Value::as_str),
        Some("smoke")
    );
    assert_eq!(
        row.get("command_proof_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/fastq-command-adapter-coverage.tsv")
    );
    assert_eq!(
        row.get("output_contract_proof_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/fastq-adapter-output-contract.tsv")
    );
    assert_eq!(
        row.get("reason").and_then(serde_json::Value::as_str),
        Some(
            "filter binding `fastq.filter_reads` / `fastp` keeps active scope, command, output, parser, expected-result, report, and schema proof for `fastq.filter_reads`",
        )
    );
    assert!(row.get("expected_outputs").and_then(serde_json::Value::as_array).is_some_and(
        |outputs| {
            outputs.iter().any(|value| value.as_str() == Some("filtered_reads_r1"))
                && outputs.iter().any(|value| value.as_str() == Some("report_json"))
        }
    ));
    assert!(row.get("required_metric_fields").and_then(serde_json::Value::as_array).is_some_and(
        |fields| {
            fields.iter().any(|value| value.as_str() == Some("reads_retained"))
                && fields.iter().any(|value| value.as_str() == Some("reads_removed_by_entropy"))
        }
    ));
}
