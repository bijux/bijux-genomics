#![cfg(feature = "bam_downstream")]
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
fn bench_readiness_tool_centric_report_tracks_named_tool_stage_lists() {
    let payload = run_cli_json(&["bench", "readiness", "render-tool-centric-report", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.tool_centric_report.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/tool-centric-report.md")
    );
    assert_eq!(payload.get("tool_count").and_then(serde_json::Value::as_u64), Some(67));
    assert_eq!(payload.get("unique_stage_count").and_then(serde_json::Value::as_u64), Some(51));
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(122));
    assert_eq!(
        payload.get("benchmark_ready_row_count").and_then(serde_json::Value::as_u64),
        Some(118)
    );
    assert_eq!(payload.get("blocked_row_count").and_then(serde_json::Value::as_u64), Some(4));
    assert_eq!(payload.get("blocked_tool_count").and_then(serde_json::Value::as_u64), Some(4));
    assert_eq!(
        payload
            .get("domain_counts")
            .and_then(|value| value.get("bam"))
            .and_then(serde_json::Value::as_u64),
        Some(49)
    );
    assert_eq!(
        payload
            .get("domain_counts")
            .and_then(|value| value.get("fastq"))
            .and_then(serde_json::Value::as_u64),
        Some(73)
    );

    let tools = payload.get("tools").and_then(serde_json::Value::as_array).expect("tools array");
    assert_eq!(tools.len(), 67);

    let samtools = tools
        .iter()
        .find(|tool| tool.get("tool_id").and_then(serde_json::Value::as_str) == Some("samtools"))
        .expect("samtools tool report");
    assert_eq!(samtools.get("stage_count").and_then(serde_json::Value::as_u64), Some(10));
    assert_eq!(samtools.get("blocked_stage_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(
        samtools.get("stages").and_then(serde_json::Value::as_array).map(|rows| {
            rows.iter()
                .filter_map(|row| row.get("stage_id").and_then(serde_json::Value::as_str))
                .collect::<Vec<_>>()
        }),
        Some(vec![
            "bam.coverage",
            "bam.duplication_metrics",
            "bam.endogenous_content",
            "bam.filter",
            "bam.length_filter",
            "bam.mapping_summary",
            "bam.mapq_filter",
            "bam.markdup",
            "bam.qc_pre",
            "bam.validate",
        ])
    );

    let fastp = tools
        .iter()
        .find(|tool| tool.get("tool_id").and_then(serde_json::Value::as_str) == Some("fastp"))
        .expect("fastp tool report");
    assert_eq!(fastp.get("stage_count").and_then(serde_json::Value::as_u64), Some(5));
    assert_eq!(fastp.get("blocked_stage_count").and_then(serde_json::Value::as_u64), Some(1));
    let fastp_blocked = fastp
        .get("stages")
        .and_then(serde_json::Value::as_array)
        .expect("fastp stages")
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.filter_low_complexity")
        })
        .expect("fastp blocked stage");
    assert_eq!(
        fastp_blocked.get("benchmark_status").and_then(serde_json::Value::as_str),
        Some("not_benchmark_ready")
    );
    assert_eq!(
        fastp_blocked.get("readiness_gap").and_then(serde_json::Value::as_str),
        Some("support")
    );

    let bowtie2 = tools
        .iter()
        .find(|tool| tool.get("tool_id").and_then(serde_json::Value::as_str) == Some("bowtie2"))
        .expect("bowtie2 tool report");
    assert_eq!(bowtie2.get("stage_count").and_then(serde_json::Value::as_u64), Some(3));
    assert_eq!(
        bowtie2
            .get("domains")
            .and_then(serde_json::Value::as_array)
            .map(|values| values.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>()),
        Some(vec!["bam", "fastq"])
    );

    let kraken2 = tools
        .iter()
        .find(|tool| tool.get("tool_id").and_then(serde_json::Value::as_str) == Some("kraken2"))
        .expect("kraken2 tool report");
    let kraken2_stage = kraken2
        .get("stages")
        .and_then(serde_json::Value::as_array)
        .and_then(|rows| rows.first())
        .expect("kraken2 stage");
    assert_eq!(
        kraken2_stage.get("report_section_id").and_then(serde_json::Value::as_str),
        Some("contamination_screening")
    );
    assert_eq!(
        kraken2_stage.get("asset_status").and_then(serde_json::Value::as_str),
        Some("assigned")
    );

    let gatk = tools
        .iter()
        .find(|tool| tool.get("tool_id").and_then(serde_json::Value::as_str) == Some("gatk"))
        .expect("gatk tool report");
    assert_eq!(gatk.get("stage_count").and_then(serde_json::Value::as_u64), Some(1));
    let gatk_stage = gatk
        .get("stages")
        .and_then(serde_json::Value::as_array)
        .and_then(|rows| rows.first())
        .expect("gatk stage");
    assert_eq!(
        gatk_stage.get("stage_id").and_then(serde_json::Value::as_str),
        Some("bam.recalibration")
    );
    assert_eq!(
        gatk_stage.get("report_section_id").and_then(serde_json::Value::as_str),
        Some("downstream_readiness")
    );
}
