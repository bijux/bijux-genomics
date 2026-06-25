#![allow(clippy::expect_used, clippy::too_many_lines)]

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
fn bench_readiness_bam_tool_scores_report_governs_real_bam_evidence() {
    let payload = run_cli_json(&["bench", "readiness", "render-bam-tool-scores", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.bam_tool_scores.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/micro/bam/BAM_TOOL_SCORES.tsv")
    );
    assert_eq!(
        payload.get("config_path").and_then(serde_json::Value::as_str),
        Some("configs/bench/local/stage-scoring.toml")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(49));
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(24));
    assert_eq!(payload.get("tool_count").and_then(serde_json::Value::as_u64), Some(25));
    assert_eq!(payload.get("scored_row_count").and_then(serde_json::Value::as_u64), Some(26));
    assert_eq!(
        payload.get("insufficient_evidence_row_count").and_then(serde_json::Value::as_u64),
        Some(23)
    );
    assert_eq!(payload.get("blocked_row_count").and_then(serde_json::Value::as_u64), Some(0));

    let failure_counts = payload
        .get("failure_class_counts")
        .and_then(serde_json::Value::as_object)
        .expect("failure_class_counts");
    assert_eq!(failure_counts.get("none").and_then(serde_json::Value::as_u64), Some(26));
    assert_eq!(
        failure_counts.get("insufficient_data").and_then(serde_json::Value::as_u64),
        Some(23)
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 49);

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
        Some("unavailable")
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
        Some("succeeded")
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
