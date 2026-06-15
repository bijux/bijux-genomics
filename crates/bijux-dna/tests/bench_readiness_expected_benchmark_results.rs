#![cfg(feature = "bam_downstream")]
#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::collections::BTreeSet;
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
fn bench_readiness_expected_benchmark_results_report_tracks_governed_result_rows() {
    let payload =
        run_cli_json(&["bench", "readiness", "render-expected-benchmark-results", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.expected_benchmark_results.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/expected-benchmark-results.tsv")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(118));
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(50));
    assert_eq!(
        payload
            .get("domain_counts")
            .and_then(|value| value.get("fastq"))
            .and_then(serde_json::Value::as_u64),
        Some(69)
    );
    assert_eq!(
        payload
            .get("domain_counts")
            .and_then(|value| value.get("bam"))
            .and_then(serde_json::Value::as_u64),
        Some(49)
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 118);

    let result_row_ids = rows
        .iter()
        .filter_map(|row| row.get("result_row_id").and_then(serde_json::Value::as_str))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        result_row_ids.len(),
        118,
        "every expected benchmark result row must keep a unique governed id"
    );
    assert!(rows.iter().all(|row| {
        row.get("stage_result_manifest_path")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|path| path.ends_with("/stage-result.json"))
    }));

    let taxonomy = rows
        .iter()
        .find(|row| {
            row.get("domain").and_then(serde_json::Value::as_str) == Some("fastq")
                && row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("fastq.screen_taxonomy")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("kraken2")
        })
        .expect("taxonomy expected-result row");
    assert_eq!(
        taxonomy.get("fixture_id").and_then(serde_json::Value::as_str),
        Some("corpus-02-edna-mini")
    );
    assert_eq!(
        taxonomy.get("sample_scope").and_then(serde_json::Value::as_str),
        Some("sample-set")
    );
    assert_eq!(
        taxonomy.get("normalized_metrics_output_id").and_then(serde_json::Value::as_str),
        Some("classification_report_json")
    );
    assert_eq!(
        taxonomy
            .get("stage_result_manifest_path")
            .and_then(serde_json::Value::as_str),
        Some(
            "runs/bench/slurm-dry-run/runs/local-benchmark-dry-run/corpus-02-edna-mini/fastq.screen_taxonomy/sample-set/kraken2/stage-result.json"
        )
    );
    let taxonomy_rows = rows
        .iter()
        .filter(|row| {
            row.get("domain").and_then(serde_json::Value::as_str) == Some("fastq")
                && row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("fastq.screen_taxonomy")
        })
        .collect::<Vec<_>>();
    assert_eq!(taxonomy_rows.len(), 4);
    for tool_id in ["centrifuge", "kaiju", "kraken2", "krakenuniq"] {
        assert!(taxonomy_rows.iter().any(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
                && row.get("fixture_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-02-edna-mini")
                && row.get("sample_scope").and_then(serde_json::Value::as_str) == Some("sample-set")
                && row.get("normalized_metrics_output_id").and_then(serde_json::Value::as_str)
                    == Some("classification_report_json")
        }));
    }

    let kinship = rows
        .iter()
        .find(|row| {
            row.get("domain").and_then(serde_json::Value::as_str) == Some("bam")
                && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.kinship")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("king")
        })
        .expect("kinship expected-result row");
    assert_eq!(
        kinship.get("fixture_id").and_then(serde_json::Value::as_str),
        Some("corpus-01-kinship-mini")
    );
    assert_eq!(kinship.get("sample_scope").and_then(serde_json::Value::as_str), Some("sample-set"));
    assert_eq!(
        kinship.get("normalized_metrics_output_id").and_then(serde_json::Value::as_str),
        Some("kinship_report")
    );
    assert!(
        kinship
            .get("expected_output_artifact_ids")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|artifacts| artifacts
                .iter()
                .any(|value| value.as_str() == Some("summary"))),
        "kinship expected-result row must retain the governed summary artifact"
    );
}
