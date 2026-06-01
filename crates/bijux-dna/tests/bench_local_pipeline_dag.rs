#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli_json(args: &[&str]) -> serde_json::Value {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(args)
        .output()
        .expect("run cli");

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
fn bench_local_pipeline_dag_validates_fastq_core_preprocess_contract() {
    let payload = run_cli_json(&["bench", "local", "validate-pipeline-dag", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_pipeline_dag_validation.v1")
    );
    assert_eq!(
        payload.get("config_path").and_then(serde_json::Value::as_str),
        Some("configs/pipelines/local/fastq-core-preprocess.toml")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/local-ready/pipeline-dag/fastq-core-preprocess.json")
    );
    assert_eq!(
        payload.get("pipeline_id").and_then(serde_json::Value::as_str),
        Some("fastq-core-preprocess")
    );
    assert_eq!(
        payload.get("default_corpus_id").and_then(serde_json::Value::as_str),
        Some("corpus-01-mini")
    );
    assert_eq!(payload.get("node_count").and_then(serde_json::Value::as_u64), Some(7));
    assert_eq!(payload.get("edge_count").and_then(serde_json::Value::as_u64), Some(12));
    assert_eq!(payload.get("acyclic").and_then(serde_json::Value::as_bool), Some(true));

    let nodes = payload.get("nodes").and_then(serde_json::Value::as_array).expect("nodes array");
    assert!(
        nodes.iter().any(|node| {
            node.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.trim_reads")
                && node.get("depends_on").and_then(serde_json::Value::as_array).is_some_and(
                    |deps| {
                        deps.iter().any(|dep| dep.as_str() == Some("fastq.validate_reads"))
                            && deps.iter().any(|dep| dep.as_str() == Some("fastq.detect_adapters"))
                    },
                )
        }),
        "trim_reads must depend on validation and adapter detection"
    );
    assert!(
        nodes.iter().any(|node| {
            node.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.report_qc")
                && node.get("upstream_inputs").and_then(serde_json::Value::as_array).is_some_and(
                    |inputs| {
                        inputs.iter().any(|input| input.as_str() == Some("validation_report"))
                            && inputs.iter().any(|input| input.as_str() == Some("filtered_profile"))
                    },
                )
        }),
        "report_qc must collate governed upstream preprocessing metrics"
    );
}

#[test]
fn bench_local_pipeline_dag_validates_fastq_paired_merge_contract() {
    let payload = run_cli_json(&[
        "bench",
        "local",
        "validate-pipeline-dag",
        "--config",
        "configs/pipelines/local/fastq-paired-merge.toml",
        "--json",
    ]);

    assert_eq!(
        payload.get("config_path").and_then(serde_json::Value::as_str),
        Some("configs/pipelines/local/fastq-paired-merge.toml")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/local-ready/pipeline-dag/fastq-paired-merge.json")
    );
    assert_eq!(
        payload.get("pipeline_id").and_then(serde_json::Value::as_str),
        Some("fastq-paired-merge")
    );
    assert_eq!(payload.get("node_count").and_then(serde_json::Value::as_u64), Some(7));
    assert_eq!(payload.get("edge_count").and_then(serde_json::Value::as_u64), Some(12));

    let nodes = payload.get("nodes").and_then(serde_json::Value::as_array).expect("nodes array");
    assert!(
        nodes.iter().any(|node| {
            node.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.merge_pairs")
                && node.get("outputs").and_then(serde_json::Value::as_array).is_some_and(
                    |outputs| {
                        outputs.iter().any(|value| value.as_str() == Some("merged_reads"))
                            && outputs
                                .iter()
                                .any(|value| value.as_str() == Some("unmerged_r1_reads"))
                            && outputs
                                .iter()
                                .any(|value| value.as_str() == Some("unmerged_r2_reads"))
                    },
                )
        }),
        "merge_pairs must expose merged and unmerged outputs in the CLI validation report"
    );
    assert!(
        nodes.iter().any(|node| {
            node.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.filter_reads")
                && node.get("upstream_inputs").and_then(serde_json::Value::as_array).is_some_and(
                    |inputs| {
                        inputs.iter().any(|value| value.as_str() == Some("merged_reads"))
                            && inputs
                                .iter()
                                .any(|value| value.as_str() == Some("unmerged_r1_reads"))
                            && inputs
                                .iter()
                                .any(|value| value.as_str() == Some("unmerged_r2_reads"))
                    },
                )
        }),
        "filter_reads must consume the merged and unmerged handoff in the CLI validation report"
    );
}

#[test]
fn bench_local_pipeline_dag_validates_fastq_edna_taxonomy_contract() {
    let payload = run_cli_json(&[
        "bench",
        "local",
        "validate-pipeline-dag",
        "--config",
        "configs/pipelines/local/fastq-edna-taxonomy.toml",
        "--json",
    ]);

    assert_eq!(
        payload.get("config_path").and_then(serde_json::Value::as_str),
        Some("configs/pipelines/local/fastq-edna-taxonomy.toml")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/local-ready/pipeline-dag/fastq-edna-taxonomy.json")
    );
    assert_eq!(
        payload.get("pipeline_id").and_then(serde_json::Value::as_str),
        Some("fastq-edna-taxonomy")
    );
    assert_eq!(
        payload.get("default_corpus_id").and_then(serde_json::Value::as_str),
        Some("corpus-02-edna-mini")
    );

    let nodes = payload.get("nodes").and_then(serde_json::Value::as_array).expect("nodes array");
    assert!(
        nodes.iter().any(|node| {
            node.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.screen_taxonomy")
                && node
                    .get("external_inputs")
                    .and_then(serde_json::Value::as_array)
                    .is_some_and(|inputs| {
                        inputs.iter().any(|value| value.as_str() == Some("taxonomy_database.root"))
                            && inputs
                                .iter()
                                .any(|value| value.as_str() == Some("taxonomy_expected_truth_table"))
                    })
                && node
                    .get("outputs")
                    .and_then(serde_json::Value::as_array)
                    .is_some_and(|outputs| {
                        outputs
                            .iter()
                            .any(|value| value.as_str() == Some("taxonomy_classification"))
                            && outputs
                                .iter()
                                .any(|value| value.as_str() == Some("unclassified_reads"))
                    })
        }),
        "screen_taxonomy must expose governed taxonomy assets plus classification and unclassified outputs"
    );
}
