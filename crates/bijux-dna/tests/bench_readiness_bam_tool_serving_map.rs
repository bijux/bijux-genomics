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
fn bench_readiness_bam_tool_serving_map_reports_governed_bam_stage_rows() {
    let payload = run_cli_json(&["bench", "readiness", "render-bam-tool-serving-map", "--json"]);
    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.bam_tool_serving_map.v1")
    );
    assert_eq!(payload.get("domain").and_then(serde_json::Value::as_str), Some("bam"));
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/bench-readiness/bam-tool-serving-map.tsv")
    );
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(24));
    assert_eq!(payload.get("tool_count").and_then(serde_json::Value::as_u64), Some(26));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(
        payload.get("row_count").and_then(serde_json::Value::as_u64),
        Some(rows.len() as u64)
    );
    assert_eq!(rows.len(), 51, "BAM readiness map must retain the governed 51-row slice");

    let has_row = |tool_id: &str,
                   stage_id: &str,
                   support_status: &str,
                   adapter_status: &str,
                   parser_status: &str,
                   corpus_status: &str| {
        rows.iter().any(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
                && row.get("stage_id").and_then(serde_json::Value::as_str) == Some(stage_id)
                && row.get("support_status").and_then(serde_json::Value::as_str)
                    == Some(support_status)
                && row.get("adapter_status").and_then(serde_json::Value::as_str)
                    == Some(adapter_status)
                && row.get("parser_status").and_then(serde_json::Value::as_str)
                    == Some(parser_status)
                && row.get("corpus_status").and_then(serde_json::Value::as_str)
                    == Some(corpus_status)
        })
    };

    assert!(
        has_row(
            "addeam",
            "bam.damage",
            "supported",
            "runnable",
            "parser_fixture_validated",
            "fixture:corpus-01-adna-damage-mini",
        ),
        "BAM readiness map must retain the governed addeam damage row"
    );
    assert!(
        has_row(
            "damageprofiler",
            "bam.damage",
            "supported",
            "runnable",
            "parser_fixture_validated",
            "fixture:corpus-01-adna-damage-mini",
        ),
        "BAM readiness map must retain the governed damageprofiler damage row"
    );
    assert!(
        has_row(
            "bwa",
            "bam.align",
            "supported",
            "runnable",
            "artifact_contract_only",
            "fixture:corpus-01-mini",
        ),
        "BAM readiness map must retain the governed bwa alignment row"
    );
    assert!(
        has_row(
            "bowtie2",
            "bam.align",
            "supported",
            "runnable",
            "artifact_contract_only",
            "fixture:corpus-01-mini",
        ),
        "BAM readiness map must retain the governed bowtie2 alignment row"
    );
    assert!(
        has_row(
            "bamtools",
            "bam.validate",
            "supported",
            "runnable",
            "parser_fixture_validated",
            "fixture:corpus-01-bam-mini",
        ),
        "BAM readiness map must retain the governed bamtools validation row"
    );
    assert!(
        has_row(
            "multiqc",
            "bam.qc_pre",
            "supported",
            "plannable",
            "parser_fixture_validated",
            "planner_only",
        ),
        "BAM readiness map must retain the governed multiqc qc_pre reporting row"
    );
    assert!(
        has_row(
            "samtools",
            "bam.bias_mitigation",
            "mismatched_contract",
            "declared_only",
            "artifact_contract_only",
            "planner_only",
        ),
        "BAM readiness map must retain the declared-only samtools bias-mitigation row"
    );
    assert!(
        has_row(
            "samtools",
            "bam.haplogroups",
            "mismatched_contract",
            "declared_only",
            "scientific_report_contract",
            "planner_only",
        ),
        "BAM readiness map must retain the declared-only samtools haplogroups row"
    );
    assert!(
        has_row(
            "yleaf",
            "bam.haplogroups",
            "supported",
            "runnable",
            "scientific_report_contract",
            "planner_only",
        ),
        "BAM readiness map must retain the governed yleaf haplogroups row"
    );
    assert!(
        !rows.iter().any(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some("samtools")
                && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.align")
        }),
        "BAM readiness map must not retain a samtools alignment row once bam.align is limited to the admitted aligners"
    );
}
