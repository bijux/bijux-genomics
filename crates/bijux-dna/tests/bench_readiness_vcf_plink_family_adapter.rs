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
fn bench_readiness_vcf_plink_adapter_reports_governed_rows() {
    let payload = run_cli_json(&["bench", "readiness", "render-vcf-plink-adapter", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_plink_adapter.v1")
    );
    assert_eq!(payload.get("domain").and_then(serde_json::Value::as_str), Some("vcf"));
    assert_eq!(payload.get("tool_id").and_then(serde_json::Value::as_str), Some("plink"));
    assert_eq!(
        payload.get("tool_status").and_then(serde_json::Value::as_str),
        Some("experimental")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/adapters/plink.vcf.json")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(
        payload.get("benchmark_ready_row_count").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(
        payload.get("missing_input_test_passed_row_count").and_then(serde_json::Value::as_u64),
        Some(2)
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 2);

    let qc_row = rows
        .iter()
        .find(|row| row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.qc"))
        .expect("plink qc row");
    assert_eq!(
        qc_row.get("benchmark_status").and_then(serde_json::Value::as_str),
        Some("benchmark_ready")
    );
    assert_eq!(
        qc_row.get("normalized_metrics_artifact_id").and_then(serde_json::Value::as_str),
        Some("qc_report")
    );
    assert_eq!(
        qc_row.get("raw_output_ids").and_then(serde_json::Value::as_array).map(|items| items.len()),
        Some(6)
    );
    assert_eq!(
        qc_row
            .get("declared_outputs")
            .and_then(serde_json::Value::as_array)
            .map(|items| items.len()),
        Some(7)
    );
    let qc_argv = qc_row
        .get("command_steps")
        .and_then(serde_json::Value::as_array)
        .and_then(|steps| steps.first())
        .and_then(|step| step.get("argv"))
        .and_then(serde_json::Value::as_array)
        .expect("plink qc argv");
    for flag in ["--missing", "--freq", "--het", "--hardy"] {
        assert!(
            qc_argv.iter().any(|item| item.as_str() == Some(flag)),
            "plink qc row must retain {flag}"
        );
    }

    let admixture_row = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.admixture")
        })
        .expect("plink admixture row");
    assert_eq!(
        admixture_row.get("benchmark_status").and_then(serde_json::Value::as_str),
        Some("not_benchmark_ready")
    );
    assert_eq!(
        admixture_row.get("normalized_metrics_artifact_id").and_then(serde_json::Value::as_str),
        Some("admixture_report")
    );
    let admixture_argv = admixture_row
        .get("command_steps")
        .and_then(serde_json::Value::as_array)
        .and_then(|steps| steps.first())
        .and_then(|step| step.get("argv"))
        .and_then(serde_json::Value::as_array)
        .expect("plink admixture argv");
    assert!(
        admixture_argv.iter().any(|item| item.as_str() == Some("--make-bed")),
        "plink admixture row must retain cohort-preparation BED output"
    );
}

#[test]
fn bench_readiness_vcf_plink2_adapter_reports_governed_rows() {
    let payload = run_cli_json(&["bench", "readiness", "render-vcf-plink2-adapter", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_plink2_adapter.v1")
    );
    assert_eq!(payload.get("domain").and_then(serde_json::Value::as_str), Some("vcf"));
    assert_eq!(payload.get("tool_id").and_then(serde_json::Value::as_str), Some("plink2"));
    assert_eq!(
        payload.get("tool_status").and_then(serde_json::Value::as_str),
        Some("experimental")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/adapters/plink2.vcf.json")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(5));
    assert_eq!(
        payload.get("benchmark_ready_row_count").and_then(serde_json::Value::as_u64),
        Some(5)
    );
    assert_eq!(
        payload.get("missing_input_test_passed_row_count").and_then(serde_json::Value::as_u64),
        Some(5)
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 5);

    let pca_row = rows
        .iter()
        .find(|row| row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.pca"))
        .expect("plink2 pca row");
    assert_eq!(
        pca_row.get("normalized_metrics_artifact_id").and_then(serde_json::Value::as_str),
        Some("pca_report")
    );
    let pca_argv = pca_row
        .get("command_steps")
        .and_then(serde_json::Value::as_array)
        .and_then(|steps| steps.first())
        .and_then(|step| step.get("argv"))
        .and_then(serde_json::Value::as_array)
        .expect("plink2 pca argv");
    assert!(
        pca_argv.iter().any(|item| item.as_str() == Some("--pca")),
        "plink2 pca row must retain eigen outputs"
    );

    let qc_row = rows
        .iter()
        .find(|row| row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.qc"))
        .expect("plink2 qc row");
    assert!(
        qc_row.get("raw_output_ids").and_then(serde_json::Value::as_array).is_some_and(|items| {
            items.iter().any(|item| item.as_str() == Some("allele_frequency_afreq"))
                && items.iter().any(|item| item.as_str() == Some("hardy_weinberg_hardy"))
        }),
        "plink2 qc row must retain AFREQ and HARDY raw outputs"
    );

    let roh_row = rows
        .iter()
        .find(|row| row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.roh"))
        .expect("plink2 roh row");
    let roh_argv = roh_row
        .get("command_steps")
        .and_then(serde_json::Value::as_array)
        .and_then(|steps| steps.first())
        .and_then(|step| step.get("argv"))
        .and_then(serde_json::Value::as_array)
        .expect("plink2 roh argv");
    assert!(
        roh_argv.iter().any(|item| item.as_str() == Some("--homozyg")),
        "plink2 roh row must retain HOM output contract"
    );

    let population_structure_row = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("vcf.population_structure")
        })
        .expect("plink2 population structure row");
    assert_eq!(
        population_structure_row
            .get("command_steps")
            .and_then(serde_json::Value::as_array)
            .map(|items| items.len()),
        Some(2),
        "population-structure row must retain prune and PCA steps"
    );
}
