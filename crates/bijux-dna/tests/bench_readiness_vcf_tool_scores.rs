#![allow(clippy::expect_used, clippy::too_many_lines)]

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli(
    sandbox: &support::RepoSandbox,
    home: &std::path::Path,
    args: &[&str],
) -> std::process::Output {
    sandbox
        .bijux_dna_command()
        .env("HOME", home)
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(args)
        .output()
        .expect("run cli")
}

fn run_cli_json(args: &[&str]) -> serde_json::Value {
    let sandbox = support::RepoSandbox::new("vcf-tool-score-readiness-").expect("repo sandbox");
    let home = tempfile::tempdir().expect("tempdir");
    sandbox.materialize_vcf_score_evidence(home.path()).expect("materialize VCF score evidence");

    let output = run_cli(&sandbox, home.path(), args);
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
fn bench_readiness_vcf_tool_scores_report_governs_real_vcf_evidence() {
    let payload = run_cli_json(&["bench", "readiness", "render-vcf-tool-scores", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_tool_scores.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/micro/vcf/VCF_TOOL_SCORES.tsv")
    );
    assert_eq!(
        payload.get("config_path").and_then(serde_json::Value::as_str),
        Some("configs/bench/local/stage-scoring.toml")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(21));
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(18));
    assert_eq!(payload.get("tool_count").and_then(serde_json::Value::as_u64), Some(6));
    assert_eq!(payload.get("scored_row_count").and_then(serde_json::Value::as_u64), Some(21));
    assert_eq!(
        payload.get("insufficient_evidence_row_count").and_then(serde_json::Value::as_u64),
        Some(0)
    );
    assert_eq!(payload.get("blocked_row_count").and_then(serde_json::Value::as_u64), Some(0));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 21);

    let failure_counts = payload
        .get("failure_class_counts")
        .and_then(serde_json::Value::as_object)
        .expect("failure_class_counts");
    assert_eq!(failure_counts.get("none").and_then(serde_json::Value::as_u64), Some(21));

    let qc_plink2 = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.qc")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("plink2")
        })
        .expect("vcf.qc plink2 row");
    assert_eq!(qc_plink2.get("score_status").and_then(serde_json::Value::as_str), Some("scored"));
    assert_eq!(
        qc_plink2.get("missingness_metric_basis").and_then(serde_json::Value::as_str),
        Some("one_minus_missingness_post")
    );
    assert_eq!(
        qc_plink2.get("phasing_imputation_metric_basis").and_then(serde_json::Value::as_str),
        Some("rsq_mean")
    );
    assert_eq!(
        qc_plink2.get("memory_source").and_then(serde_json::Value::as_str),
        Some("declared_stage_tool_resource")
    );

    let imputation_metrics = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("vcf.imputation_metrics")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("beagle")
        })
        .expect("vcf.imputation_metrics beagle row");
    assert_eq!(
        imputation_metrics.get("score_status").and_then(serde_json::Value::as_str),
        Some("scored")
    );
    assert_eq!(
        imputation_metrics.get("truth_correctness_basis").and_then(serde_json::Value::as_str),
        Some("concordance")
    );
    assert_eq!(
        imputation_metrics
            .get("phasing_imputation_metric_basis")
            .and_then(serde_json::Value::as_str),
        Some("dosage_r2")
    );
    assert_eq!(
        imputation_metrics.get("failure_class").and_then(serde_json::Value::as_str),
        Some("none")
    );

    let phasing = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.phasing")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("shapeit5")
        })
        .expect("vcf.phasing shapeit5 row");
    assert_eq!(
        phasing.get("phasing_imputation_metric_basis").and_then(serde_json::Value::as_str),
        Some("phased_genotype_fraction")
    );
    assert_eq!(
        phasing.get("runtime_source").and_then(serde_json::Value::as_str),
        Some("not_available")
    );

    let pca_eigensoft = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.pca")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("eigensoft")
        })
        .expect("vcf.pca eigensoft row");
    assert_eq!(
        pca_eigensoft.get("population_metric_basis").and_then(serde_json::Value::as_str),
        Some("variant_count")
    );
}
