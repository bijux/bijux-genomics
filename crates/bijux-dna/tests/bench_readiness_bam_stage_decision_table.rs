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
        Some("target/bench-readiness/bam-stage-decision-table.tsv")
    );
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(24));
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(24));

    let decision_counts = payload
        .get("decision_counts")
        .and_then(serde_json::Value::as_object)
        .expect("decision_counts object");
    assert_eq!(
        decision_counts.get("benchmark_ready").and_then(serde_json::Value::as_u64),
        Some(16)
    );
    assert!(
        decision_counts.get("needs_corpus").is_none(),
        "the governed BAM stage decision table should no longer carry needs_corpus rows"
    );
    assert_eq!(decision_counts.get("needs_parser").and_then(serde_json::Value::as_u64), Some(6));
    assert_eq!(
        decision_counts.get("future_not_in_hpc_round").and_then(serde_json::Value::as_u64),
        Some(2)
    );
    assert!(
        decision_counts.get("needs_adapter").is_none(),
        "the governed BAM stage decision table currently carries no needs_adapter rows"
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 24);
    assert!(
        rows.iter().any(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.validate")
                && row.get("decision").and_then(serde_json::Value::as_str)
                    == Some("benchmark_ready")
                && row.get("primary_tool_id").and_then(serde_json::Value::as_str)
                    == Some("samtools")
                && row.get("selected_tool_id").and_then(serde_json::Value::as_str)
                    == Some("samtools")
                && row.get("corpus_status").and_then(serde_json::Value::as_str)
                    == Some("fixture:corpus-01-bam-mini")
        }),
        "bam.validate must remain benchmark_ready through the samtools fixture-backed row"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.damage")
                && row.get("decision").and_then(serde_json::Value::as_str)
                    == Some("benchmark_ready")
                && row.get("primary_tool_id").and_then(serde_json::Value::as_str)
                    == Some("mapdamage2")
                && row.get("selected_tool_id").and_then(serde_json::Value::as_str)
                    == Some("mapdamage2")
                && row.get("parser_status").and_then(serde_json::Value::as_str)
                    == Some("parser_fixture_validated")
                && row.get("corpus_status").and_then(serde_json::Value::as_str)
                    == Some("fixture:corpus-01-adna-damage-mini")
        }),
        "bam.damage must remain benchmark_ready through the governed mapdamage2 damage row"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.authenticity")
                && row.get("decision").and_then(serde_json::Value::as_str)
                    == Some("benchmark_ready")
                && row.get("primary_tool_id").and_then(serde_json::Value::as_str)
                    == Some("authenticct")
                && row.get("selected_tool_id").and_then(serde_json::Value::as_str)
                    == Some("authenticct")
                && row.get("parser_status").and_then(serde_json::Value::as_str)
                    == Some("parser_fixture_validated")
                && row.get("corpus_status").and_then(serde_json::Value::as_str)
                    == Some("fixture:corpus-01-bam-mini")
        }),
        "bam.authenticity must now be benchmark_ready through the governed authenticct advisory row"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.complexity")
                && row.get("decision").and_then(serde_json::Value::as_str)
                    == Some("benchmark_ready")
                && row.get("primary_tool_id").and_then(serde_json::Value::as_str)
                    == Some("preseq")
                && row.get("selected_tool_id").and_then(serde_json::Value::as_str)
                    == Some("preseq")
                && row.get("support_status").and_then(serde_json::Value::as_str)
                    == Some("supported")
                && row.get("adapter_status").and_then(serde_json::Value::as_str)
                    == Some("runnable")
                && row.get("parser_status").and_then(serde_json::Value::as_str)
                    == Some("parser_fixture_validated")
                && row.get("corpus_status").and_then(serde_json::Value::as_str)
                    == Some("fixture:corpus-01-bam-mini")
        }),
        "bam.complexity must now be benchmark_ready through the governed preseq complexity-projection row"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.qc_pre")
                && row.get("decision").and_then(serde_json::Value::as_str)
                    == Some("benchmark_ready")
                && row.get("primary_tool_id").and_then(serde_json::Value::as_str) == Some("multiqc")
                && row.get("selected_tool_id").and_then(serde_json::Value::as_str)
                    == Some("multiqc")
                && row.get("parser_status").and_then(serde_json::Value::as_str)
                    == Some("parser_fixture_validated")
                && row.get("corpus_status").and_then(serde_json::Value::as_str)
                    == Some("fixture:corpus-01-bam-mini")
        }),
        "bam.qc_pre must now be benchmark_ready through the governed multiqc fixture-backed row"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("bam.mapping_summary")
                && row.get("decision").and_then(serde_json::Value::as_str)
                    == Some("benchmark_ready")
                && row.get("primary_tool_id").and_then(serde_json::Value::as_str)
                    == Some("samtools")
                && row.get("selected_tool_id").and_then(serde_json::Value::as_str)
                    == Some("samtools")
                && row.get("parser_status").and_then(serde_json::Value::as_str)
                    == Some("parser_fixture_validated")
                && row.get("corpus_status").and_then(serde_json::Value::as_str)
                    == Some("fixture:corpus-01-bam-mini")
        }),
        "bam.mapping_summary must now be benchmark_ready through the governed samtools partial-mapping row"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.mapq_filter")
                && row.get("decision").and_then(serde_json::Value::as_str)
                    == Some("benchmark_ready")
                && row.get("primary_tool_id").and_then(serde_json::Value::as_str)
                    == Some("samtools")
                && row.get("selected_tool_id").and_then(serde_json::Value::as_str)
                    == Some("samtools")
                && row.get("parser_status").and_then(serde_json::Value::as_str)
                    == Some("parser_fixture_validated")
                && row.get("corpus_status").and_then(serde_json::Value::as_str)
                    == Some("fixture:corpus-01-bam-mini")
        }),
        "bam.mapq_filter must now be benchmark_ready through the governed samtools MAPQ-threshold row"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.filter")
                && row.get("decision").and_then(serde_json::Value::as_str)
                    == Some("benchmark_ready")
                && row.get("primary_tool_id").and_then(serde_json::Value::as_str)
                    == Some("samtools")
                && row.get("selected_tool_id").and_then(serde_json::Value::as_str)
                    == Some("samtools")
                && row.get("parser_status").and_then(serde_json::Value::as_str)
                    == Some("parser_fixture_validated")
                && row.get("corpus_status").and_then(serde_json::Value::as_str)
                    == Some("fixture:corpus-01-bam-mini")
        }),
        "bam.filter must now be benchmark_ready through the governed samtools mixed-filter row"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.markdup")
                && row.get("decision").and_then(serde_json::Value::as_str)
                    == Some("benchmark_ready")
                && row.get("primary_tool_id").and_then(serde_json::Value::as_str) == Some("picard")
                && row.get("selected_tool_id").and_then(serde_json::Value::as_str) == Some("picard")
                && row.get("parser_status").and_then(serde_json::Value::as_str)
                    == Some("parser_fixture_validated")
                && row.get("corpus_status").and_then(serde_json::Value::as_str)
                    == Some("fixture:corpus-01-bam-mini")
        }),
        "bam.markdup must now be benchmark_ready through the governed picard duplicate-cluster row"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("bam.length_filter")
                && row.get("decision").and_then(serde_json::Value::as_str)
                    == Some("benchmark_ready")
                && row.get("primary_tool_id").and_then(serde_json::Value::as_str)
                    == Some("samtools")
                && row.get("selected_tool_id").and_then(serde_json::Value::as_str)
                    == Some("samtools")
                && row.get("parser_status").and_then(serde_json::Value::as_str)
                    == Some("parser_fixture_validated")
                && row.get("corpus_status").and_then(serde_json::Value::as_str)
                    == Some("fixture:corpus-01-bam-mini")
        }),
        "bam.length_filter must now be benchmark_ready through the governed samtools length-threshold row"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("bam.duplication_metrics")
                && row.get("decision").and_then(serde_json::Value::as_str)
                    == Some("benchmark_ready")
                && row.get("primary_tool_id").and_then(serde_json::Value::as_str)
                    == Some("samtools")
                && row.get("selected_tool_id").and_then(serde_json::Value::as_str)
                    == Some("samtools")
                && row.get("parser_status").and_then(serde_json::Value::as_str)
                    == Some("parser_fixture_validated")
                && row.get("corpus_status").and_then(serde_json::Value::as_str)
                    == Some("fixture:corpus-01-bam-mini")
        }),
        "bam.duplication_metrics must now be benchmark_ready through the governed samtools duplicate-cluster row"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.coverage")
                && row.get("decision").and_then(serde_json::Value::as_str)
                    == Some("benchmark_ready")
                && row.get("primary_tool_id").and_then(serde_json::Value::as_str)
                    == Some("mosdepth")
                && row.get("selected_tool_id").and_then(serde_json::Value::as_str)
                    == Some("mosdepth")
                && row.get("parser_status").and_then(serde_json::Value::as_str)
                    == Some("parser_fixture_validated")
                && row.get("corpus_status").and_then(serde_json::Value::as_str)
                    == Some("fixture:corpus-01-bam-mini")
        }),
        "bam.coverage must now be benchmark_ready through the governed mosdepth target-window row"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.gc_bias")
                && row.get("decision").and_then(serde_json::Value::as_str)
                    == Some("benchmark_ready")
                && row.get("primary_tool_id").and_then(serde_json::Value::as_str) == Some("picard")
                && row.get("selected_tool_id").and_then(serde_json::Value::as_str) == Some("picard")
                && row.get("parser_status").and_then(serde_json::Value::as_str)
                    == Some("parser_fixture_validated")
                && row.get("corpus_status").and_then(serde_json::Value::as_str)
                    == Some("fixture:corpus-01-bam-mini")
        }),
        "bam.gc_bias must now be benchmark_ready through the governed picard GC-window ladder row"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.insert_size")
                && row.get("decision").and_then(serde_json::Value::as_str)
                    == Some("benchmark_ready")
                && row.get("primary_tool_id").and_then(serde_json::Value::as_str)
                    == Some("picard")
                && row.get("selected_tool_id").and_then(serde_json::Value::as_str)
                    == Some("picard")
                && row.get("parser_status").and_then(serde_json::Value::as_str)
                    == Some("parser_fixture_validated")
                && row.get("corpus_status").and_then(serde_json::Value::as_str)
                    == Some("fixture:corpus-01-bam-mini")
        }),
        "bam.insert_size must now be benchmark_ready through the governed picard insert-size triplet row"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("bam.endogenous_content")
                && row.get("decision").and_then(serde_json::Value::as_str)
                    == Some("benchmark_ready")
                && row.get("primary_tool_id").and_then(serde_json::Value::as_str)
                    == Some("samtools")
                && row.get("selected_tool_id").and_then(serde_json::Value::as_str)
                    == Some("samtools")
                && row.get("parser_status").and_then(serde_json::Value::as_str)
                    == Some("parser_fixture_validated")
                && row.get("corpus_status").and_then(serde_json::Value::as_str)
                    == Some("fixture:corpus-01-bam-mini")
        }),
        "bam.endogenous_content must now be benchmark_ready through the governed samtools endogenous-partial-mapping row"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("bam.overlap_correction")
                && row.get("decision").and_then(serde_json::Value::as_str)
                    == Some("benchmark_ready")
                && row.get("primary_tool_id").and_then(serde_json::Value::as_str)
                    == Some("bamutil")
                && row.get("selected_tool_id").and_then(serde_json::Value::as_str)
                    == Some("bamutil")
                && row.get("adapter_status").and_then(serde_json::Value::as_str)
                    == Some("runnable")
                && row.get("parser_status").and_then(serde_json::Value::as_str)
                    == Some("parser_fixture_validated")
                && row.get("corpus_status").and_then(serde_json::Value::as_str)
                    == Some("fixture:corpus-01-bam-mini")
        }),
        "bam.overlap_correction must now be benchmark_ready through the governed bamutil paired-overlap row"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.align")
                && row.get("decision").and_then(serde_json::Value::as_str) == Some("needs_parser")
                && row.get("primary_tool_id").and_then(serde_json::Value::as_str) == Some("bwa")
                && row.get("selected_tool_id").and_then(serde_json::Value::as_str) == Some("bwa")
                && row.get("parser_status").and_then(serde_json::Value::as_str)
                    == Some("artifact_contract_only")
        }),
        "bam.align must stay parser-blocked until a normalized BAM parser is fixture-validated"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("bam.bias_mitigation")
                && row.get("decision").and_then(serde_json::Value::as_str)
                    == Some("needs_parser")
                && row.get("primary_tool_id").and_then(serde_json::Value::as_str)
                    == Some("samtools")
                && row.get("selected_tool_id").and_then(serde_json::Value::as_str)
                    == Some("mapdamage2")
                && row.get("support_status").and_then(serde_json::Value::as_str)
                    == Some("supported")
        }),
        "bam.bias_mitigation must expose the fallback mapdamage2 row while the primary samtools row is not eligible"
    );
    for stage_id in ["bam.genotyping", "bam.recalibration"] {
        assert!(
            rows.iter().any(|row| {
                row.get("stage_id").and_then(serde_json::Value::as_str) == Some(stage_id)
                    && row.get("decision").and_then(serde_json::Value::as_str)
                        == Some("future_not_in_hpc_round")
                    && row.get("primary_tool_id").is_some_and(serde_json::Value::is_null)
            }),
            "{stage_id} must remain explicitly future_not_in_hpc_round until it enters the governed BAM benchmark registry"
        );
    }
}
