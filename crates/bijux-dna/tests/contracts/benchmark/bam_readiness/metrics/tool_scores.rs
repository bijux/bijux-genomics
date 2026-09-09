#![allow(clippy::expect_used, clippy::too_many_lines)]

use crate::support;

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

fn run_cli_json(args: &[&str]) -> serde_json::Value {
    let sandbox = support::RepoSandbox::new("bam-tool-score-readiness-").expect("repo sandbox");
    let home = tempfile::tempdir().expect("tempdir");
    sandbox.materialize_bam_score_evidence(home.path()).expect("materialize BAM score evidence");
    let output = run_cli(&sandbox, args);
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
fn bench_readiness_bam_tool_scores_report_governs_real_bam_evidence() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let payload = run_cli_json(&["bench", "readiness", "render-bam-tool-scores", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.bam_tool_scores.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("artifacts/benchmark-smoke/micro/bam/BAM_TOOL_SCORES.tsv")
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
    assert_eq!(payload.get("blocked_row_count").and_then(serde_json::Value::as_u64), Some(3));

    let failure_counts = payload
        .get("failure_class_counts")
        .and_then(serde_json::Value::as_object)
        .expect("failure_class_counts");
    assert_eq!(
        failure_counts.values().filter_map(serde_json::Value::as_u64).sum::<u64>(),
        rows.len() as u64
    );
    assert_eq!(
        failure_counts.get("blocked_external_access").and_then(serde_json::Value::as_u64),
        Some(3)
    );
    for (stage_id, tool_id) in
        [("bam.damage", "ngsbriggs"), ("bam.kinship", "angsd"), ("bam.sex", "angsd")]
    {
        let row = rows
            .iter()
            .find(|row| {
                row.get("stage_id").and_then(serde_json::Value::as_str) == Some(stage_id)
                    && row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
            })
            .expect("licensed BAM score row");
        assert_eq!(row.get("score_status").and_then(serde_json::Value::as_str), Some("blocked"));
        assert_eq!(
            row.get("failure_class").and_then(serde_json::Value::as_str),
            Some("blocked_external_access")
        );
    }

    let coverage_samtools = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.coverage")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("samtools")
        })
        .expect("bam.coverage samtools row");
    assert_eq!(
        coverage_samtools.get("score_status").and_then(serde_json::Value::as_str),
        Some("scored")
    );
    assert_eq!(
        coverage_samtools.get("truth_correctness_basis").and_then(serde_json::Value::as_str),
        Some("mean_breadth_1x")
    );
    assert_eq!(
        coverage_samtools.get("coverage_metric_basis").and_then(serde_json::Value::as_str),
        Some("mean_depth")
    );
    assert_eq!(
        coverage_samtools.get("truth_correctness_score").and_then(serde_json::Value::as_f64),
        Some(0.875)
    );

    let contamination_verifybamid2 = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.contamination")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("verifybamid2")
        })
        .expect("bam.contamination verifybamid2 row");
    assert_eq!(
        contamination_verifybamid2.get("score_status").and_then(serde_json::Value::as_str),
        Some("scored")
    );
    assert_eq!(
        contamination_verifybamid2.get("failure_class").and_then(serde_json::Value::as_str),
        Some("none")
    );
    assert_eq!(
        contamination_verifybamid2
            .get("micro_execution_status")
            .and_then(serde_json::Value::as_str),
        None
    );
    assert_eq!(
        contamination_verifybamid2
            .get("truth_correctness_basis")
            .and_then(serde_json::Value::as_str),
        Some("one_minus_contamination_estimate")
    );

    let align_bowtie2 = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.align")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bowtie2")
        })
        .expect("bam.align bowtie2 row");
    assert_eq!(
        align_bowtie2.get("score_status").and_then(serde_json::Value::as_str),
        Some("insufficient_evidence")
    );
    assert_eq!(
        align_bowtie2.get("failure_class").and_then(serde_json::Value::as_str),
        Some("insufficient_data")
    );
    assert_eq!(
        align_bowtie2.get("micro_execution_status").and_then(serde_json::Value::as_str),
        None
    );

    let gc_bias_picard = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.gc_bias")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("picard")
        })
        .expect("bam.gc_bias picard row");
    assert_eq!(
        gc_bias_picard.get("score_status").and_then(serde_json::Value::as_str),
        Some("scored")
    );
    assert_eq!(
        gc_bias_picard.get("correctness_signal").and_then(serde_json::Value::as_str),
        Some("output_contract")
    );
    assert_eq!(
        gc_bias_picard.get("contract_correctness_score").and_then(serde_json::Value::as_f64),
        Some(1.0)
    );
}
