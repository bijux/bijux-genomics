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
fn bench_readiness_fastq_adapter_output_contract_reports_governed_rows() {
    let payload =
        run_cli_json(&["bench", "readiness", "render-fastq-adapter-output-contract", "--json"]);
    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.fastq_adapter_output_contract.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/bench-readiness/fastq-adapter-output-contract.tsv")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(75));
    assert_eq!(payload.get("adapter_row_count").and_then(serde_json::Value::as_u64), Some(68));
    assert_eq!(
        payload.get("complete_adapter_row_count").and_then(serde_json::Value::as_u64),
        Some(68)
    );
    assert_eq!(
        payload.get("incomplete_adapter_row_count").and_then(serde_json::Value::as_u64),
        Some(0)
    );
    assert_eq!(
        payload.get("missing_adapter_row_count").and_then(serde_json::Value::as_u64),
        Some(7)
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 75, "report must retain the governed FASTQ 75-row slice");
    assert!(
        rows.iter().any(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some("seqkit_stats")
                && row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("fastq.profile_reads")
                && row.get("output_contract_status").and_then(serde_json::Value::as_str)
                    == Some("complete")
                && row.get("normalized_metrics_output_id").and_then(serde_json::Value::as_str)
                    == Some("qc_json")
        }),
        "report must retain the governed seqkit_stats profile-reads contract row"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some("fastqc")
                && row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("fastq.detect_adapters")
                && row.get("output_contract_status").and_then(serde_json::Value::as_str)
                    == Some("complete")
                && row
                    .get("stage_expected_artifact_ids")
                    .and_then(serde_json::Value::as_array)
                    .is_some_and(|artifacts| {
                        artifacts.iter().any(|value| value == "report_json")
                            && artifacts.iter().any(|value| value == "adapter_report")
                            && artifacts.iter().any(|value| value == "adapter_evidence_dir")
                    })
        }),
        "report must retain the governed detect-adapters contract row for fastqc"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bowtie2")
                && row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("fastq.deplete_host")
                && row.get("output_contract_status").and_then(serde_json::Value::as_str)
                    == Some("complete")
                && row.get("normalized_metrics_output_id").and_then(serde_json::Value::as_str)
                    == Some("host_depletion_report_json")
        }),
        "report must retain the governed bowtie2 host-depletion contract row"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some("diamond")
                && row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("fastq.screen_taxonomy")
                && row.get("output_contract_status").and_then(serde_json::Value::as_str)
                    == Some("missing_adapter")
        }),
        "report must keep the planned diamond taxonomy row explicit as missing an adapter"
    );
}
