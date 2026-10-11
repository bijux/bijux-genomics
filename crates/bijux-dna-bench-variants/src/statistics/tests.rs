#![cfg(test)]
use super::{
    admission::resolve_governed_vcf_stats_smoke_contract, fixtures::write_governed_stats_input_vcf,
    truth::summarize_variant_truth,
};
fn inputs() -> crate::inputs::VariantBenchmarkInputs {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates")
        .parent()
        .expect("repository");
    let matrix: bijux_dna_bench_model::LocalVcfStageMatrixConfig = toml::from_str(
        &std::fs::read_to_string(root.join("configs/bench/local/vcf-stage-matrix.toml"))
            .expect("matrix"),
    )
    .expect("parse matrix");
    bijux_dna_bench_model::validate_local_vcf_stage_matrix_config(&matrix).expect("admit matrix");
    let rows = matrix.rows.into_iter().filter(|row| row.stage_id == "vcf.stats").collect();
    crate::inputs::VariantBenchmarkInputs::new(rows, Vec::new()).expect("statistics inputs")
}

#[test]
fn governed_stats_contract_uses_cohort_matrix_row() {
    let contract =
        resolve_governed_vcf_stats_smoke_contract(&inputs(), "bcftools").expect("resolve contract");
    assert_eq!(contract.stage_id, "vcf.stats");
    assert_eq!(contract.tool_id, "bcftools");
    assert_eq!(contract.corpus_id, "vcf_production_regression");
    assert_eq!(contract.input_fixture_id, "stats_cohort_minimal");
    assert_eq!(contract.sample_name, "cohort_stats");
}

#[test]
fn governed_stats_fixture_matches_expected_truth() {
    let dir = tempfile::tempdir().expect("tempdir");
    let input_vcf = dir.path().join("stats_input.vcf");
    write_governed_stats_input_vcf(&input_vcf).expect("write governed input");
    let truth = summarize_variant_truth(&input_vcf).expect("summarize truth");
    assert_eq!(truth.variants, 4);
    assert_eq!(truth.snps, 3);
    assert_eq!(truth.indels, 1);
    assert_eq!(truth.transitions, 2);
    assert_eq!(truth.transversions, 1);
    assert_eq!(truth.samples, 2);
}

#[test]
fn unknown_tool_refuses_before_statistics_effects() -> anyhow::Result<()> {
    let root = tempfile::tempdir()?;
    assert!(super::run_local_vcf_stats_smoke(&inputs(), root.path(), "unregistered").is_err());
    assert_eq!(std::fs::read_dir(root.path())?.count(), 0);
    Ok(())
}
#[test]
fn statistics_refuses_unowned_metrics_without_replacing_them() -> anyhow::Result<()> {
    let root = tempfile::tempdir()?;
    let output = root.path().join("artifacts/runs/bench/local-smoke/vcf.stats/bcftools");
    std::fs::create_dir_all(&output)?;
    let metrics = output.join("metrics.json");
    std::fs::write(&metrics, b"user statistics evidence")?;
    assert!(super::run_local_vcf_stats_smoke(&inputs(), root.path(), "bcftools").is_err());
    assert_eq!(std::fs::read(metrics)?, b"user statistics evidence");
    Ok(())
}
#[test]
fn statistics_publication_retains_native_execution_evidence() -> anyhow::Result<()> {
    let root = tempfile::tempdir()?;
    let inputs = inputs();
    let first = super::run_local_vcf_stats_smoke(&inputs, root.path(), "bcftools")?;
    let output = root.path().join(&first.output_root);
    let executions = output.join("executions");
    let first_run = std::fs::read_dir(&executions)?.next().expect("retained execution")?.path();
    let manifest_path = first_run.join("stage-result.json");
    let original_manifest = std::fs::read(&manifest_path)?;
    let second = super::run_local_vcf_stats_smoke(&inputs, root.path(), "bcftools")?;
    assert_eq!(first.variant_count, 4);
    assert_eq!(second.variant_count, 4);
    assert!((second.ti_tv - 2.0).abs() < f64::EPSILON);
    assert_eq!(std::fs::read_dir(&executions)?.count(), 2);
    assert_eq!(std::fs::read(&manifest_path)?, original_manifest);
    assert!(root.path().join(&second.stats_json_path).is_file());
    assert!(root.path().join(&second.bcftools_stats_path).is_file());
    let retained: bijux_dna_bench_model::BenchmarkStageResultManifest =
        serde_json::from_slice(&original_manifest)?;
    for artifact in retained.outputs {
        assert!(root.path().join(artifact.realized_path).starts_with(&first_run));
    }
    let stable: bijux_dna_bench_model::BenchmarkStageResultManifest = serde_json::from_slice(
        &std::fs::read(root.path().join(second.stage_result_manifest_path))?,
    )?;
    for artifact in stable.outputs {
        let path = root.path().join(artifact.realized_path);
        assert!(path.starts_with(&output));
        assert!(path.is_file());
    }
    Ok(())
}
