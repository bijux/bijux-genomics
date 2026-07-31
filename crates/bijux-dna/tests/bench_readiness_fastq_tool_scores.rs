#![allow(clippy::expect_used, clippy::too_many_lines)]

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

const FASTQ_SCORE_EVIDENCE_COMMANDS: &[&[&str]] = &[
    &["bench", "local", "materialize-stage", "--stage-id", "fastq.filter_reads"],
    &["bench", "local", "materialize-stage", "--stage-id", "fastq.trim_reads"],
    &["bench", "local", "materialize-stage", "--stage-id", "fastq.validate_reads"],
    &["bench", "local", "run-edna-micro-pipeline"],
];

fn run_cli(sandbox: &support::RepoSandbox, args: &[&str]) -> std::process::Output {
    let home = tempfile::tempdir().expect("tempdir");

    sandbox
        .bijux_dna_command()
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(args)
        .output()
        .expect("run cli")
}

fn materialize_fastq_score_evidence(sandbox: &support::RepoSandbox) {
    for args in FASTQ_SCORE_EVIDENCE_COMMANDS {
        let output = run_cli(sandbox, args);
        assert!(
            output.status.success(),
            "FASTQ score evidence command failed: {}\ncommand: {}\nstdout:\n{}\nstderr:\n{}",
            output.status,
            args.join(" "),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

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
        Some("runs/bench/micro/fastq/FASTQ_TOOL_SCORES.tsv")
    );
    assert_eq!(
        payload.get("config_path").and_then(serde_json::Value::as_str),
        Some("configs/bench/local/stage-scoring.toml")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(71));
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(27));
    assert_eq!(payload.get("tool_count").and_then(serde_json::Value::as_u64), Some(42));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 71);
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
    assert_eq!(scored_row_count, 10);
    assert_eq!(insufficient_evidence_row_count, 61);
    assert_eq!(blocked_row_count, 0);
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
    assert_eq!(failure_counts.get("none").and_then(serde_json::Value::as_u64), Some(10));
    assert_eq!(
        failure_counts.get("insufficient_data").and_then(serde_json::Value::as_u64),
        Some(61)
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
        Some("evidence_report")
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
