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
fn bench_readiness_missing_benchmark_pairs_reports_governed_gaps() {
    let payload = run_cli_json(&["bench", "readiness", "render-missing-benchmark-pairs", "--json"]);
    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.missing_benchmark_pairs.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/bench-readiness/missing-benchmark-pairs.tsv")
    );
    assert_eq!(payload.get("missing_pair_count").and_then(serde_json::Value::as_u64), Some(6));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(false));

    let domain_counts = payload
        .get("domain_counts")
        .and_then(serde_json::Value::as_object)
        .expect("domain_counts object");
    assert_eq!(
        domain_counts.get("bam").and_then(serde_json::Value::as_u64),
        Some(6),
        "the current missing benchmark-pair slice must be entirely BAM-owned"
    );
    assert!(
        domain_counts.get("fastq").is_none(),
        "FASTQ currently has no missing benchmark pairs in this governed slice"
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 6, "the governed missing-pair slice must retain six BAM rows");
    assert!(
        rows.iter().any(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.align")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("samtools")
                && row.get("support_status").and_then(serde_json::Value::as_str) == Some("planned")
        }),
        "bam.align / samtools must remain visible as a planned missing benchmark pair"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.authenticity")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("damageprofiler")
                && row.get("support_status").and_then(serde_json::Value::as_str)
                    == Some("supported")
        }),
        "bam.authenticity / damageprofiler must remain visible as a missing benchmark pair"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.damage")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("addeam")
                && row.get("support_status").and_then(serde_json::Value::as_str)
                    == Some("supported")
        }),
        "bam.damage / addeam must remain visible as a missing benchmark pair"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("bam.overlap_correction")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("samtools")
                && row.get("support_status").and_then(serde_json::Value::as_str) == Some("planned")
        }),
        "bam.overlap_correction / samtools must remain visible as a planned missing pair"
    );
    assert!(
        !rows.iter().any(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.filter")
        }),
        "bam.filter must stay out of the missing benchmark-pair report once all admitted tools are covered"
    );
    assert!(
        !rows.iter().any(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.mapq_filter")
        }),
        "bam.mapq_filter must stay out of the missing benchmark-pair report once all admitted tools are covered"
    );
    assert!(
        !rows.iter().any(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.length_filter")
        }),
        "bam.length_filter must stay out of the missing benchmark-pair report once all admitted tools are covered"
    );
    assert!(
        !rows.iter().any(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("bam.duplication_metrics")
        }),
        "bam.duplication_metrics must stay out of the missing benchmark-pair report once all admitted tools are covered"
    );
    assert!(
        !rows.iter().any(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.complexity")
        }),
        "bam.complexity must stay out of the missing benchmark-pair report while its planned preseq row already exists in the benchmark matrix"
    );
    assert!(
        !rows.iter().any(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.markdup")
        }),
        "bam.markdup must stay out of the missing benchmark-pair report once all admitted tools are covered"
    );
}
