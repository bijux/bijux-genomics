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
fn bench_local_vcf_pca_smoke_reports_complete_cohort_contract() {
    let payload = run_cli_json(&["bench", "local", "run-vcf-pca-smoke", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_vcf_pca_smoke.v1")
    );
    assert_eq!(
        payload.get("command").and_then(serde_json::Value::as_str),
        Some("bijux-dna bench local run-vcf-pca-smoke --tool-id plink2")
    );
    assert_eq!(payload.get("stage_id").and_then(serde_json::Value::as_str), Some("vcf.pca"));
    assert_eq!(payload.get("tool_id").and_then(serde_json::Value::as_str), Some("plink2"));
    assert_eq!(
        payload.get("corpus_id").and_then(serde_json::Value::as_str),
        Some("vcf_production_regression")
    );
    assert_eq!(
        payload.get("input_fixture_id").and_then(serde_json::Value::as_str),
        Some("vcf_mini_multisample_cohort")
    );
    assert_eq!(
        payload.get("fixture_manifest_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/tests/fixtures/corpora/vcf-mini/manifest.toml")
    );
    assert_eq!(
        payload.get("input_vcf_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.pca/plink2/artifacts/input/pca_input.vcf")
    );
    assert_eq!(
        payload.get("sample_metadata_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.pca/plink2/artifacts/input/sample_metadata.tsv")
    );
    assert_eq!(
        payload.get("population_metadata_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.pca/plink2/artifacts/input/population_metadata.tsv")
    );
    assert_eq!(
        payload.get("population_labels_manifest_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.pca/plink2/artifacts/input/population_labels.json")
    );
    assert_eq!(
        payload.get("output_root").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.pca/plink2")
    );
    assert_eq!(
        payload.get("pca_tsv_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.pca/plink2/pca.tsv")
    );
    assert_eq!(
        payload.get("pca_json_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.pca/plink2/pca.json")
    );
    assert_eq!(
        payload.get("source_eigenvec_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.pca/plink2/source_eigenvec.tsv")
    );
    assert_eq!(
        payload.get("source_eigenval_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.pca/plink2/source_eigenval.tsv")
    );
    assert_eq!(
        payload.get("source_pca_manifest_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.pca/plink2/source_pca_manifest.json")
    );
    assert_eq!(
        payload.get("source_logs_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.pca/plink2/source_logs.txt")
    );
    assert_eq!(
        payload.get("stage_result_manifest_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.pca/plink2/stage-result.json")
    );
    assert_eq!(payload.get("exit_code").and_then(serde_json::Value::as_i64), Some(0));
    assert_eq!(payload.get("variant_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(payload.get("sample_count").and_then(serde_json::Value::as_u64), Some(4));
    assert_eq!(
        payload
            .get("excluded_samples")
            .and_then(serde_json::Value::as_array)
            .map(|rows| rows.len()),
        Some(0)
    );
    assert_eq!(
        payload
            .get("unexpected_samples")
            .and_then(serde_json::Value::as_array)
            .map(|rows| rows.len()),
        Some(0)
    );

    let execution_mode =
        payload.get("execution_mode").and_then(serde_json::Value::as_str).expect("execution_mode");
    assert!(matches!(execution_mode, "real_tool" | "fallback_proxy"));
    assert_eq!(
        payload.get("tool_ok").and_then(serde_json::Value::as_bool),
        Some(execution_mode == "real_tool")
    );

    let eigenvalues = payload
        .get("eigenvalues")
        .and_then(serde_json::Value::as_array)
        .expect("eigenvalues array");
    assert!(eigenvalues.len() >= 2, "expected at least two PCA eigenvalues");

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 4);
    assert_eq!(
        rows.iter()
            .filter_map(|row| row.get("sample_id").and_then(serde_json::Value::as_str))
            .collect::<Vec<_>>(),
        vec!["sample_a", "sample_b", "sample_c", "sample_d"]
    );
    assert_eq!(
        rows[0].get("population_id").and_then(serde_json::Value::as_str),
        Some("cohort_alpha")
    );
    assert_eq!(
        rows[0].get("population_label").and_then(serde_json::Value::as_str),
        Some("Cohort Alpha")
    );
    assert_eq!(rows[0].get("sex").and_then(serde_json::Value::as_str), Some("female"));
    assert!(rows.iter().all(|row| row.get("pc1").and_then(serde_json::Value::as_f64).is_some()));
    assert!(rows.iter().all(|row| row.get("pc2").and_then(serde_json::Value::as_f64).is_some()));

    let repo_root = support::repo_root().expect("repo root");
    let persisted_path = repo_root.join("target/local-smoke/vcf.pca/plink2/pca.json");
    let persisted_raw = std::fs::read_to_string(&persisted_path).expect("read persisted report");
    let persisted: serde_json::Value = serde_json::from_str(&persisted_raw).expect("parse report");
    assert_eq!(
        persisted.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_vcf_pca_smoke.v1")
    );
    assert_eq!(persisted.get("sample_count").and_then(serde_json::Value::as_u64), Some(4));
}
