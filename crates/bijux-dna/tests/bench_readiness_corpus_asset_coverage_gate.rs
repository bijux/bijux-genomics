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
fn bench_readiness_corpus_asset_coverage_gate_reports_complete_benchmark_rows() {
    let payload =
        run_cli_json(&["bench", "readiness", "render-corpus-asset-coverage-gate", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.corpus_asset_coverage_gate.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/gate-corpus-assets-complete.json")
    );
    assert_eq!(payload.get("passes_gate"), Some(&serde_json::Value::Bool(true)));
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(123));
    assert_eq!(
        payload.get("benchmark_ready_row_count").and_then(serde_json::Value::as_u64),
        Some(116)
    );
    assert_eq!(payload.get("gate_row_count").and_then(serde_json::Value::as_u64), Some(116));
    assert_eq!(payload.get("gate_passed_row_count").and_then(serde_json::Value::as_u64), Some(116));
    assert_eq!(payload.get("gate_failed_row_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("excluded_row_count").and_then(serde_json::Value::as_u64), Some(7));
    assert_eq!(
        payload.get("benchmark_ready_asset_required_row_count").and_then(serde_json::Value::as_u64),
        Some(18)
    );
    assert_eq!(
        payload.get("benchmark_ready_asset_assigned_row_count").and_then(serde_json::Value::as_u64),
        Some(18)
    );
    assert_eq!(
        payload.get("benchmark_ready_asset_missing_row_count").and_then(serde_json::Value::as_u64),
        Some(0)
    );
    assert_eq!(
        payload
            .get("domain_row_counts")
            .and_then(|value| value.get("fastq"))
            .and_then(serde_json::Value::as_u64),
        Some(74)
    );
    assert_eq!(
        payload
            .get("domain_row_counts")
            .and_then(|value| value.get("bam"))
            .and_then(serde_json::Value::as_u64),
        Some(49)
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 123);
    assert!(rows
        .iter()
        .all(|row| { row.get("gate_status").and_then(serde_json::Value::as_str) != Some("fail") }));

    let taxonomy = rows
        .iter()
        .find(|row| {
            row.get("domain").and_then(serde_json::Value::as_str) == Some("fastq")
                && row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("fastq.screen_taxonomy")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("kraken2")
        })
        .expect("taxonomy gate row");
    assert_eq!(
        taxonomy.get("gate_scope").and_then(serde_json::Value::as_str),
        Some("benchmark_submission")
    );
    assert_eq!(
        taxonomy.get("corpus_assignment_status").and_then(serde_json::Value::as_str),
        Some("assigned")
    );
    assert_eq!(
        taxonomy.get("asset_assignment_status").and_then(serde_json::Value::as_str),
        Some("assigned")
    );

    let kinship = rows
        .iter()
        .find(|row| {
            row.get("domain").and_then(serde_json::Value::as_str) == Some("bam")
                && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.kinship")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("king")
        })
        .expect("kinship gate row");
    assert_eq!(
        kinship.get("gate_scope").and_then(serde_json::Value::as_str),
        Some("benchmark_submission")
    );
    assert_eq!(
        kinship.get("asset_assignment_status").and_then(serde_json::Value::as_str),
        Some("assigned")
    );
    assert!(
        kinship.get("assigned_assets").and_then(serde_json::Value::as_array).is_some_and(
            |assets| {
                assets.iter().any(|value| {
                    value.as_str() == Some("reference_panel=human_like_relatedness_panel")
                })
            }
        ),
        "kinship gate row must carry the governed relatedness panel asset"
    );

    let trim_reads = rows
        .iter()
        .find(|row| {
            row.get("domain").and_then(serde_json::Value::as_str) == Some("fastq")
                && row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("fastq.trim_reads")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("trimmomatic")
        })
        .expect("trim-reads gate row");
    assert_eq!(
        trim_reads.get("asset_assignment_status").and_then(serde_json::Value::as_str),
        Some("not_required")
    );

    let excluded_index = rows
        .iter()
        .find(|row| {
            row.get("domain").and_then(serde_json::Value::as_str) == Some("fastq")
                && row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("fastq.index_reference")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bowtie2_build")
        })
        .expect("excluded index-reference row");
    assert_eq!(
        excluded_index.get("gate_scope").and_then(serde_json::Value::as_str),
        Some("excluded")
    );
    assert_eq!(
        excluded_index.get("asset_assignment_status").and_then(serde_json::Value::as_str),
        Some("assigned")
    );
}
