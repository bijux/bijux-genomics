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
fn bench_readiness_fastq_report_map_reports_governed_expected_result_bindings() {
    let payload = run_cli_json(&["bench", "readiness", "render-fastq-report-map", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.fastq_report_map.v1")
    );
    assert_eq!(payload.get("domain").and_then(serde_json::Value::as_str), Some("fastq"));
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/fastq/fastq-report-map.tsv")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(69));
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(26));
    assert_eq!(payload.get("tool_count").and_then(serde_json::Value::as_u64), Some(41));
    assert_eq!(payload.get("section_count").and_then(serde_json::Value::as_u64), Some(6));
    assert_eq!(payload.get("summary_table_count").and_then(serde_json::Value::as_u64), Some(7));
    assert_eq!(
        payload
            .get("section_counts")
            .and_then(|value| value.get("quality_profiling"))
            .and_then(serde_json::Value::as_u64),
        Some(13)
    );
    assert_eq!(
        payload
            .get("section_counts")
            .and_then(|value| value.get("read_cleanup"))
            .and_then(serde_json::Value::as_u64),
        Some(37)
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 69);

    let index_reference = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.index_reference")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bowtie2_build")
        })
        .expect("index reference row");
    assert_eq!(
        index_reference.get("support_status").and_then(serde_json::Value::as_str),
        Some("supported")
    );
    assert_eq!(
        index_reference.get("report_section_id").and_then(serde_json::Value::as_str),
        Some("reference_preparation")
    );
    assert_eq!(
        index_reference.get("summary_table_id").and_then(serde_json::Value::as_str),
        Some("reference_index_assets")
    );

    let estimate_complexity = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.estimate_library_complexity_prealign")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bijux_dna")
        })
        .expect("estimate library complexity row");
    assert_eq!(
        estimate_complexity.get("support_status").and_then(serde_json::Value::as_str),
        Some("supported")
    );
    assert_eq!(
        estimate_complexity.get("summary_table_id").and_then(serde_json::Value::as_str),
        Some("premerge_complexity")
    );

    let profile_reads = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.profile_reads")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("seqkit_stats")
        })
        .expect("profile reads row");
    assert_eq!(
        profile_reads.get("report_section_id").and_then(serde_json::Value::as_str),
        Some("quality_profiling")
    );
    assert_eq!(
        profile_reads.get("summary_table_id").and_then(serde_json::Value::as_str),
        Some("qc_signal_profiles")
    );
    assert_eq!(
        profile_reads.get("produces_reports_only").and_then(serde_json::Value::as_bool),
        Some(true)
    );

    let infer_asvs = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.infer_asvs")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("dada2")
        })
        .expect("infer asvs row");
    assert_eq!(
        infer_asvs.get("criticality").and_then(serde_json::Value::as_str),
        Some("experimental")
    );
    assert_eq!(
        infer_asvs.get("report_section_id").and_then(serde_json::Value::as_str),
        Some("amplicon_interpretation")
    );
    assert_eq!(
        infer_asvs.get("summary_table_id").and_then(serde_json::Value::as_str),
        Some("amplicon_features")
    );
}
