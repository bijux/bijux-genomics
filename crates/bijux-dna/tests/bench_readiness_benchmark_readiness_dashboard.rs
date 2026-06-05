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
fn bench_readiness_benchmark_readiness_dashboard_tracks_governed_summary_counts() {
    let payload =
        run_cli_json(&["bench", "readiness", "render-benchmark-readiness-dashboard", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.benchmark_readiness_dashboard.v1")
    );
    assert_eq!(
        payload.get("markdown_output_path").and_then(serde_json::Value::as_str),
        Some("target/bench-readiness/FASTQ_BAM_BENCHMARK_READINESS.md")
    );
    assert_eq!(
        payload.get("json_output_path").and_then(serde_json::Value::as_str),
        Some("target/bench-readiness/FASTQ_BAM_BENCHMARK_READINESS.json")
    );
    assert_eq!(payload.get("expected_pair_count").and_then(serde_json::Value::as_u64), Some(123));
    assert_eq!(payload.get("ready_pair_count").and_then(serde_json::Value::as_u64), Some(112));
    assert_eq!(payload.get("blocked_pair_count").and_then(serde_json::Value::as_u64), Some(11));
    assert_eq!(
        payload
            .get("blocker_counts")
            .and_then(|value| value.get("corpus"))
            .and_then(serde_json::Value::as_u64),
        Some(6)
    );
    assert_eq!(
        payload
            .get("blocker_counts")
            .and_then(|value| value.get("support"))
            .and_then(serde_json::Value::as_u64),
        Some(5)
    );

    let matrix = payload.get("matrix").expect("matrix summary");
    assert_eq!(matrix.get("stage_count").and_then(serde_json::Value::as_u64), Some(51));
    assert_eq!(matrix.get("tool_count").and_then(serde_json::Value::as_u64), Some(67));
    assert_eq!(
        matrix.get("surface_status").and_then(serde_json::Value::as_str),
        Some("attention_required")
    );

    let adapters = payload.get("adapters").expect("adapter summary");
    assert_eq!(
        adapters.get("attention_required_pair_count").and_then(serde_json::Value::as_u64),
        Some(5)
    );
    assert_eq!(
        adapters
            .get("status_counts")
            .and_then(|value| value.get("declared_only"))
            .and_then(serde_json::Value::as_u64),
        Some(5)
    );
    assert_eq!(
        adapters
            .get("status_counts")
            .and_then(|value| value.get("runnable"))
            .and_then(serde_json::Value::as_u64),
        Some(118)
    );

    let parsers = payload.get("parsers").expect("parser summary");
    assert_eq!(
        parsers.get("benchmark_reporting_pair_count").and_then(serde_json::Value::as_u64),
        Some(112)
    );
    assert_eq!(parsers.get("blocked_pair_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(parsers.get("excluded_pair_count").and_then(serde_json::Value::as_u64), Some(11));

    let corpora = payload.get("corpora").expect("corpus summary");
    assert_eq!(corpora.get("corpus_family_count").and_then(serde_json::Value::as_u64), Some(7));
    assert_eq!(corpora.get("assigned_stage_count").and_then(serde_json::Value::as_u64), Some(48));
    assert_eq!(corpora.get("blocked_pair_count").and_then(serde_json::Value::as_u64), Some(6));
    assert_eq!(
        corpora
            .get("corpus_family_ids")
            .and_then(serde_json::Value::as_array)
            .map(|values| values.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>()),
        Some(vec![
            "corpus-01",
            "corpus-01-adna-bam",
            "corpus-01-bam",
            "corpus-01-genotyping",
            "corpus-01-kinship",
            "corpus-02",
            "corpus-03",
        ])
    );

    let assets = payload.get("assets").expect("asset summary");
    assert_eq!(
        assets.get("asset_required_pair_count").and_then(serde_json::Value::as_u64),
        Some(18)
    );
    assert_eq!(assets.get("ready_pair_count").and_then(serde_json::Value::as_u64), Some(18));
    assert_eq!(assets.get("blocked_pair_count").and_then(serde_json::Value::as_u64), Some(0));

    let reports = payload.get("reports").expect("report summary");
    assert_eq!(reports.get("output_count").and_then(serde_json::Value::as_u64), Some(5));
    assert_eq!(
        reports.get("expected_result_row_count").and_then(serde_json::Value::as_u64),
        Some(112)
    );
    assert_eq!(reports.get("stage_section_count").and_then(serde_json::Value::as_u64), Some(51));
    assert_eq!(reports.get("tool_section_count").and_then(serde_json::Value::as_u64), Some(67));
    assert_eq!(reports.get("corpus_section_count").and_then(serde_json::Value::as_u64), Some(7));

    let report_outputs = payload
        .get("report_outputs")
        .and_then(serde_json::Value::as_array)
        .expect("report outputs");
    assert_eq!(report_outputs.len(), 5);
    assert!(report_outputs.iter().any(|row| {
        row.get("report_id").and_then(serde_json::Value::as_str) == Some("corpus_centric_report")
            && row.get("item_count").and_then(serde_json::Value::as_u64) == Some(7)
    }));

    let blocked_pairs =
        payload.get("blocked_pairs").and_then(serde_json::Value::as_array).expect("blocked pairs");
    assert_eq!(blocked_pairs.len(), 11);
    assert!(blocked_pairs.iter().any(|row| {
        row.get("row_id").and_then(serde_json::Value::as_str)
            == Some("fastq:fastq.index_reference:bowtie2_build")
            && row.get("readiness_gap").and_then(serde_json::Value::as_str) == Some("corpus")
    }));
    assert!(blocked_pairs.iter().any(|row| {
        row.get("row_id").and_then(serde_json::Value::as_str)
            == Some("fastq:fastq.trim_reads:seqpurge")
            && row.get("readiness_gap").and_then(serde_json::Value::as_str) == Some("support")
    }));
    assert!(blocked_pairs.iter().any(|row| {
        row.get("row_id").and_then(serde_json::Value::as_str)
            == Some("fastq:fastq.report_qc:multiqc")
            && row.get("corpus_status").and_then(serde_json::Value::as_str) == Some("planner_only")
    }));
}
