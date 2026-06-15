#![allow(clippy::expect_used, clippy::too_many_lines)]

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
fn bench_readiness_vcf_eigensoft_adapter_reports_governed_rows() {
    let payload = run_cli_json(&["bench", "readiness", "render-vcf-eigensoft-adapter", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_eigensoft_adapter.v1")
    );
    assert_eq!(payload.get("domain").and_then(serde_json::Value::as_str), Some("vcf"));
    assert_eq!(payload.get("tool_id").and_then(serde_json::Value::as_str), Some("eigensoft"));
    assert_eq!(
        payload.get("tool_status").and_then(serde_json::Value::as_str),
        Some("experimental")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/adapters/eigensoft.vcf.json")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(
        payload.get("benchmark_ready_row_count").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(payload.get("parser_output_row_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(
        payload.get("conversion_output_row_count").and_then(serde_json::Value::as_u64),
        Some(2)
    );
    assert_eq!(payload.get("pca_output_row_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(
        payload.get("missing_input_test_passed_row_count").and_then(serde_json::Value::as_u64),
        Some(2)
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 2);

    let pca_row = rows
        .iter()
        .find(|row| row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.pca"))
        .expect("eigensoft pca row");
    assert_eq!(
        pca_row.get("benchmark_status").and_then(serde_json::Value::as_str),
        Some("benchmark_ready")
    );
    assert_eq!(
        pca_row.get("normalized_metrics_artifact_id").and_then(serde_json::Value::as_str),
        Some("pca_report")
    );
    assert!(
        pca_row.get("raw_output_ids").and_then(serde_json::Value::as_array).is_some_and(|items| {
            items.iter().any(|item| item.as_str() == Some("eigensoft_geno"))
                && items.iter().any(|item| item.as_str() == Some("eigensoft_snp"))
                && items.iter().any(|item| item.as_str() == Some("eigensoft_ind"))
                && items.iter().any(|item| item.as_str() == Some("smartpca_eigenvec"))
                && items.iter().any(|item| item.as_str() == Some("smartpca_eigenval"))
        }),
        "pca row must retain conversion and smartpca raw outputs"
    );
    assert!(
        pca_row.get("command_steps").and_then(serde_json::Value::as_array).is_some_and(|steps| {
            steps.iter().any(|step| {
                step.get("argv").and_then(serde_json::Value::as_array).is_some_and(|argv| {
                    argv.iter()
                        .any(|part| part.as_str().is_some_and(|value| value.contains("convertf")))
                })
            })
        }),
        "pca row must retain convertf command rendering"
    );

    let population_structure_row = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("vcf.population_structure")
        })
        .expect("eigensoft population structure row");
    assert_eq!(
        population_structure_row
            .get("normalized_metrics_artifact_id")
            .and_then(serde_json::Value::as_str),
        Some("population_structure_report")
    );
    assert!(
        population_structure_row
            .get("command_steps")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|steps| {
                steps.iter().any(|step| {
                    step.get("argv").and_then(serde_json::Value::as_array).is_some_and(|argv| {
                        argv.iter().any(|part| {
                            part.as_str().is_some_and(|value| value.contains("smartpca"))
                        })
                    })
                })
            }),
        "population structure row must retain smartpca command rendering"
    );
}
