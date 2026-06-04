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
fn bench_readiness_bam_adapter_output_contract_reports_governed_rows() {
    let payload =
        run_cli_json(&["bench", "readiness", "render-bam-adapter-output-contract", "--json"]);
    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.bam_adapter_output_contract.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/bench-readiness/bam-adapter-output-contract.tsv")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(51));
    assert_eq!(payload.get("adapter_row_count").and_then(serde_json::Value::as_u64), Some(48));
    assert_eq!(
        payload.get("complete_adapter_row_count").and_then(serde_json::Value::as_u64),
        Some(48)
    );
    assert_eq!(
        payload.get("incomplete_adapter_row_count").and_then(serde_json::Value::as_u64),
        Some(0)
    );
    assert_eq!(
        payload.get("missing_adapter_row_count").and_then(serde_json::Value::as_u64),
        Some(3)
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 51, "report must retain the governed BAM 51-row slice");
    assert!(
        rows.iter().any(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some("samtools")
                && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.validate")
                && row.get("output_contract_status").and_then(serde_json::Value::as_str)
                    == Some("complete")
                && row.get("normalized_metrics_output_id").and_then(serde_json::Value::as_str)
                    == Some("validation_report")
        }),
        "report must retain the governed samtools validate contract row"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bowtie2")
                && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.align")
                && row.get("output_contract_status").and_then(serde_json::Value::as_str)
                    == Some("complete")
                && row.get("normalized_metrics_output_id").and_then(serde_json::Value::as_str)
                    == Some("align_metrics")
        }),
        "report must retain the governed bowtie2 alignment contract row"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some("mapdamage2")
                && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.damage")
                && row.get("output_contract_status").and_then(serde_json::Value::as_str)
                    == Some("complete")
                && row.get("normalized_metrics_output_id").and_then(serde_json::Value::as_str)
                    == Some("damage_report")
        }),
        "report must retain the governed mapdamage2 damage contract row"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bamutil")
                && row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("bam.overlap_correction")
                && row.get("adapter_status").and_then(serde_json::Value::as_str)
                    == Some("runnable")
                && row.get("output_contract_status").and_then(serde_json::Value::as_str)
                    == Some("complete")
                && row.get("normalized_metrics_output_id").and_then(serde_json::Value::as_str)
                    == Some("summary")
        }),
        "report must retain the governed bamutil overlap-correction contract row"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bcftools")
                && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.genotyping")
                && row.get("output_contract_status").and_then(serde_json::Value::as_str)
                    == Some("missing_adapter")
        }),
        "report must keep the planned bcftools genotyping row explicit as missing an adapter"
    );
}
