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
fn bench_readiness_bam_stage_decision_table_reports_governed_bam_stage_outcomes() {
    let payload =
        run_cli_json(&["bench", "readiness", "render-bam-stage-decision-table", "--json"]);
    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.bam_stage_decision_table.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/bam-stage-decision-table.tsv")
    );
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(24));
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(24));

    let decision_counts = payload
        .get("decision_counts")
        .and_then(serde_json::Value::as_object)
        .expect("decision_counts object");
    assert_eq!(
        decision_counts.get("benchmark_ready").and_then(serde_json::Value::as_u64),
        Some(24)
    );
    assert!(
        decision_counts.get("needs_corpus").is_none(),
        "the governed BAM stage decision table should no longer carry needs_corpus rows"
    );
    assert!(decision_counts.get("needs_parser").is_none());
    assert!(decision_counts.get("future_not_in_hpc_round").is_none());
    assert!(
        decision_counts.get("needs_adapter").is_none(),
        "the governed BAM stage decision table currently carries no needs_adapter rows"
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 24);
    let assert_stage_row = |stage_id: &str, tool_id: &str, corpus_status: &str| {
        assert!(
            rows.iter().any(|row| {
                row.get("stage_id").and_then(serde_json::Value::as_str) == Some(stage_id)
                    && row.get("decision").and_then(serde_json::Value::as_str)
                        == Some("benchmark_ready")
                    && row.get("primary_tool_id").and_then(serde_json::Value::as_str)
                        == Some(tool_id)
                    && row.get("selected_tool_id").and_then(serde_json::Value::as_str)
                        == Some(tool_id)
                    && row.get("support_status").and_then(serde_json::Value::as_str)
                        == Some("supported")
                    && row.get("adapter_status").and_then(serde_json::Value::as_str)
                        == Some("runnable")
                    && row.get("parser_status").and_then(serde_json::Value::as_str)
                        == Some("parser_fixture_validated")
                    && row.get("corpus_status").and_then(serde_json::Value::as_str)
                        == Some(corpus_status)
            }),
            "{stage_id} must remain benchmark_ready through the governed {tool_id} fixture-backed row"
        );
    };

    for (stage_id, tool_id, corpus_status) in [
        ("bam.align", "bwa", "fixture:corpus-01-mini"),
        (
            "bam.authenticity",
            "authenticct",
            "fixture:corpus-01-adna-damage-mini",
        ),
        (
            "bam.bias_mitigation",
            "mapdamage2",
            "fixture:corpus-01-bam-mini",
        ),
        ("bam.complexity", "preseq", "fixture:corpus-01-bam-mini"),
        (
            "bam.contamination",
            "schmutzi",
            "fixture:corpus-01-adna-bam-mini",
        ),
        ("bam.coverage", "mosdepth", "fixture:corpus-01-bam-mini"),
        (
            "bam.damage",
            "mapdamage2",
            "fixture:corpus-01-adna-damage-mini",
        ),
        (
            "bam.duplication_metrics",
            "samtools",
            "fixture:corpus-01-bam-mini",
        ),
        (
            "bam.endogenous_content",
            "samtools",
            "fixture:corpus-01-bam-mini",
        ),
        ("bam.filter", "samtools", "fixture:corpus-01-bam-mini"),
        ("bam.gc_bias", "picard", "fixture:corpus-01-bam-mini"),
        (
            "bam.genotyping",
            "angsd",
            "fixture:corpus-01-genotyping-mini",
        ),
        (
            "bam.haplogroups",
            "yleaf",
            "fixture:corpus-01-adna-bam-mini",
        ),
        ("bam.insert_size", "picard", "fixture:corpus-01-bam-mini"),
        ("bam.kinship", "king", "fixture:corpus-01-kinship-mini"),
        (
            "bam.length_filter",
            "samtools",
            "fixture:corpus-01-bam-mini",
        ),
        (
            "bam.mapping_summary",
            "samtools",
            "fixture:corpus-01-bam-mini",
        ),
        ("bam.mapq_filter", "samtools", "fixture:corpus-01-bam-mini"),
        ("bam.markdup", "samtools", "fixture:corpus-01-bam-mini"),
        (
            "bam.overlap_correction",
            "bamutil",
            "fixture:corpus-01-bam-mini",
        ),
        ("bam.qc_pre", "samtools", "fixture:corpus-01-bam-mini"),
        (
            "bam.recalibration",
            "gatk",
            "fixture:corpus-01-bam-mini",
        ),
        ("bam.sex", "rxy", "fixture:corpus-01-adna-bam-mini"),
        ("bam.validate", "samtools", "fixture:corpus-01-bam-mini"),
    ] {
        assert_stage_row(stage_id, tool_id, corpus_status);
    }
}
