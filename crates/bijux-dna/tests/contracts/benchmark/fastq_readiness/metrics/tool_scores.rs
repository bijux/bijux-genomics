#![allow(clippy::expect_used, clippy::too_many_lines)]

use crate::support;

use super::score_evidence::{materialize_fastq_score_evidence, run_cli};

fn run_cli_json(sandbox: &support::RepoSandbox, args: &[&str]) -> serde_json::Value {
    let output = run_cli(sandbox, args);
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
fn bench_readiness_fastq_tool_scores_report_governs_real_fastq_evidence() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let sandbox = support::RepoSandbox::new("fastq-tool-scores-").expect("repo sandbox");
    materialize_fastq_score_evidence(&sandbox);
    let payload =
        run_cli_json(&sandbox, &["bench", "readiness", "render-fastq-tool-scores", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.fastq_tool_scores.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("artifacts/benchmark-smoke/micro/fastq/FASTQ_TOOL_SCORES.tsv")
    );
    assert_eq!(
        payload.get("config_path").and_then(serde_json::Value::as_str),
        Some("configs/bench/local/stage-scoring.toml")
    );
    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(
        payload.get("row_count").and_then(serde_json::Value::as_u64),
        Some(rows.len() as u64)
    );
    assert!(rows.iter().all(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str)
            != Some("fastq.profile_read_lengths")
            || matches!(
                row.get("tool_id").and_then(serde_json::Value::as_str),
                Some("seqfu" | "seqkit")
            )
    }));
    let score_status_count = |status: &str| {
        u64::try_from(
            rows.iter()
                .filter(|row| {
                    row.get("score_status").and_then(serde_json::Value::as_str) == Some(status)
                })
                .count(),
        )
        .expect("score status count fits u64")
    };
    let scored_row_count = score_status_count("scored");
    let insufficient_evidence_row_count = score_status_count("insufficient_evidence");
    let blocked_row_count = score_status_count("blocked");
    assert_eq!(scored_row_count, 5);
    assert_eq!(insufficient_evidence_row_count, 65);
    assert_eq!(blocked_row_count, 1);
    for (stage, tool) in [
        ("fastq.normalize_primers", "cutadapt"),
        ("fastq.remove_chimeras", "vsearch"),
        ("fastq.infer_asvs", "dada2"),
        ("fastq.cluster_otus", "vsearch"),
        ("fastq.normalize_abundance", "seqkit"),
    ] {
        let row = rows
            .iter()
            .find(|row| row["stage_id"] == stage && row["tool_id"] == tool)
            .expect("retained amplicon binding remains visible");
        assert_eq!(row["score_status"], "insufficient_evidence");
        for field in [
            "score_total",
            "truth_correctness_score",
            "contract_correctness_score",
            "retained_reads",
            "dropped_reads",
        ] {
            assert!(
                row.get(field).is_some_and(serde_json::Value::is_null),
                "{stage}/{tool}: fabricated {field}"
            );
        }
    }

    assert_eq!(
        payload.get("scored_row_count").and_then(serde_json::Value::as_u64),
        Some(scored_row_count)
    );
    assert_eq!(
        payload.get("insufficient_evidence_row_count").and_then(serde_json::Value::as_u64),
        Some(insufficient_evidence_row_count)
    );
    assert_eq!(
        payload.get("blocked_row_count").and_then(serde_json::Value::as_u64),
        Some(blocked_row_count)
    );

    let failure_counts = payload
        .get("failure_class_counts")
        .and_then(serde_json::Value::as_object)
        .expect("failure_class_counts");
    assert_eq!(
        failure_counts.values().filter_map(serde_json::Value::as_u64).sum::<u64>(),
        u64::try_from(rows.len()).expect("row count fits u64")
    );
    assert_eq!(
        failure_counts.get("blocked_external_access").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    let fastuniq = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.remove_duplicates")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("fastuniq")
        })
        .expect("fastq.remove_duplicates fastuniq row");
    assert_eq!(fastuniq.get("score_status").and_then(serde_json::Value::as_str), Some("blocked"));
    assert_eq!(
        fastuniq.get("failure_class").and_then(serde_json::Value::as_str),
        Some("blocked_external_access")
    );
    let filter_fastp = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.filter_reads")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("fastp")
        })
        .expect("fastq.filter_reads fastp row");
    assert_eq!(
        filter_fastp.get("score_status").and_then(serde_json::Value::as_str),
        Some("scored")
    );
    assert_eq!(
        filter_fastp.get("truth_correctness_basis").and_then(serde_json::Value::as_str),
        Some("retained_fraction")
    );
    assert_eq!(filter_fastp.get("retained_reads").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(filter_fastp.get("dropped_reads").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(
        filter_fastp.get("memory_source").and_then(serde_json::Value::as_str),
        Some("not_available")
    );
    assert_eq!(filter_fastp.get("failure_class").and_then(serde_json::Value::as_str), Some("none"));

    let validate_fastqvalidator = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.validate_reads")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("fastqvalidator")
        })
        .expect("fastq.validate_reads fastqvalidator row");
    assert_eq!(
        validate_fastqvalidator.get("score_status").and_then(serde_json::Value::as_str),
        Some("scored")
    );
    assert_eq!(
        validate_fastqvalidator.get("truth_correctness_basis").and_then(serde_json::Value::as_str),
        Some("validation_pass_fraction")
    );
    assert_eq!(
        validate_fastqvalidator.get("truth_correctness_score").and_then(serde_json::Value::as_f64),
        Some(1.0)
    );
    assert_eq!(
        validate_fastqvalidator.get("retained_reads").and_then(serde_json::Value::as_u64),
        Some(6)
    );

    let screen_taxonomy = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.screen_taxonomy")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("kraken2")
        })
        .expect("fastq.screen_taxonomy kraken2 row");
    assert_eq!(
        screen_taxonomy.get("score_status").and_then(serde_json::Value::as_str),
        Some("scored")
    );
    assert_eq!(
        screen_taxonomy.get("truth_correctness_basis").and_then(serde_json::Value::as_str),
        Some("classified_fraction")
    );
    assert_eq!(screen_taxonomy.get("retained_reads").and_then(serde_json::Value::as_u64), Some(4));
    assert_eq!(screen_taxonomy.get("dropped_reads").and_then(serde_json::Value::as_u64), Some(0));

    let trim_alientrimmer = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.trim_reads")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("alientrimmer")
        })
        .expect("fastq.trim_reads alientrimmer row");
    assert_eq!(
        trim_alientrimmer.get("score_status").and_then(serde_json::Value::as_str),
        Some("insufficient_evidence")
    );
    assert_eq!(
        trim_alientrimmer.get("failure_class").and_then(serde_json::Value::as_str),
        Some("insufficient_data")
    );

    let trim_fastp = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.trim_reads")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("fastp")
        })
        .expect("fastq.trim_reads fastp row");
    assert_eq!(trim_fastp.get("score_status").and_then(serde_json::Value::as_str), Some("scored"));
    assert_eq!(trim_fastp.get("retained_reads").and_then(serde_json::Value::as_u64), Some(6));
    assert_eq!(trim_fastp.get("dropped_reads").and_then(serde_json::Value::as_u64), Some(0));
}

#[test]
fn bench_readiness_fastq_filter_score_requires_materialized_evidence() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let sandbox = support::RepoSandbox::new("fastq-tool-scores-absent-").expect("repo sandbox");
    let payload =
        run_cli_json(&sandbox, &["bench", "readiness", "render-fastq-tool-scores", "--json"]);
    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("score rows");
    let fastp = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.filter_reads")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("fastp")
        })
        .expect("fastp filter score");
    assert_eq!(
        fastp.get("score_status").and_then(serde_json::Value::as_str),
        Some("insufficient_evidence")
    );
    assert!(fastp.get("score_total").is_some_and(serde_json::Value::is_null));
}
