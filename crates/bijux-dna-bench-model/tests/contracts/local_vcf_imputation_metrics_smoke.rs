use bijux_dna_bench_model::{
    benchmark_local_vcf_imputation_metrics_smoke_schema,
    validate_benchmark_local_vcf_imputation_metrics_smoke_report,
    BenchmarkImputationQualityFieldAvailability, BenchmarkLocalVcfImputationMetricsSmokeReport,
    BENCHMARK_LOCAL_VCF_IMPUTATION_METRICS_SMOKE_SCHEMA_V1,
};

fn report() -> BenchmarkLocalVcfImputationMetricsSmokeReport {
    BenchmarkLocalVcfImputationMetricsSmokeReport {
        schema_version: BENCHMARK_LOCAL_VCF_IMPUTATION_METRICS_SMOKE_SCHEMA_V1.to_string(),
        command: "bijux-dna bench local run-vcf-imputation-metrics-smoke --tool-id impute5"
            .to_string(),
        stage_id: "vcf.imputation_metrics".to_string(),
        tool_id: "impute5".to_string(),
        corpus_id: "masked-truth".to_string(),
        input_fixture_id: "masked-truth-sample".to_string(),
        panel_id: "reference-panel".to_string(),
        map_id: "genetic-map".to_string(),
        output_root: "artifacts/runs/bench/local-smoke/vcf.imputation_metrics/impute5".to_string(),
        imputation_metrics_path:
            "artifacts/runs/bench/local-smoke/vcf.imputation_metrics/impute5/imputation_metrics.json"
                .to_string(),
        source_imputation_qc_path:
            "artifacts/runs/bench/local-smoke/vcf.imputation_metrics/impute5/source_imputation_qc.json"
                .to_string(),
        source_impute_smoke_metrics_path:
            "artifacts/runs/bench/local-smoke/vcf.imputation_metrics/impute5/source_impute_smoke_metrics.json"
                .to_string(),
        source_imputation_manifest_path:
            "artifacts/runs/bench/local-smoke/vcf.imputation_metrics/impute5/source_imputation_manifest.json"
                .to_string(),
        stage_result_manifest_path:
            "artifacts/runs/bench/local-smoke/vcf.imputation_metrics/impute5/stage-result.json".to_string(),
        started_at: "unix:1".to_string(),
        finished_at: "unix:2".to_string(),
        elapsed_seconds: 0.25,
        exit_code: 0,
        concordance: Some(0.98),
        mean_info_score: 0.91,
        r2_available: true,
        dosage_r2: Some(0.95),
        low_confidence_sites: 2,
        masked_truth_sites: 100,
        quality_field_availability: BenchmarkImputationQualityFieldAvailability {
            concordance: true,
            dosage_r2: true,
            maf_strata: true,
            masked_truth_sites: true,
        },
        missing_quality_fields: Vec::new(),
        status: "complete".to_string(),
    }
}

#[test]
fn imputation_metrics_smoke_contract_accepts_complete_and_explicit_missing_evidence(
) -> anyhow::Result<()> {
    let complete = report();
    validate_benchmark_local_vcf_imputation_metrics_smoke_report(&complete)?;

    let mut missing = complete;
    missing.concordance = None;
    missing.dosage_r2 = None;
    missing.r2_available = false;
    missing.masked_truth_sites = 0;
    missing.quality_field_availability = BenchmarkImputationQualityFieldAvailability {
        concordance: false,
        dosage_r2: false,
        maf_strata: false,
        masked_truth_sites: false,
    };
    missing.missing_quality_fields =
        ["concordance", "dosage_r2", "maf_strata", "masked_truth_sites"]
            .into_iter()
            .map(str::to_string)
            .collect();
    missing.status = "explicit_missing_quality_fields".to_string();
    validate_benchmark_local_vcf_imputation_metrics_smoke_report(&missing)?;
    Ok(())
}

#[test]
fn imputation_metrics_smoke_contract_rejects_identity_metric_and_status_drift() {
    let mut wrong_stage = report();
    wrong_stage.stage_id = "vcf.impute".to_string();
    assert!(validate_benchmark_local_vcf_imputation_metrics_smoke_report(&wrong_stage).is_err());

    let mut invalid_metric = report();
    invalid_metric.mean_info_score = 1.1;
    assert!(validate_benchmark_local_vcf_imputation_metrics_smoke_report(&invalid_metric).is_err());

    let mut availability_drift = report();
    availability_drift.r2_available = false;
    assert!(
        validate_benchmark_local_vcf_imputation_metrics_smoke_report(&availability_drift).is_err()
    );

    let mut status_drift = report();
    status_drift.status = "explicit_missing_quality_fields".to_string();
    assert!(validate_benchmark_local_vcf_imputation_metrics_smoke_report(&status_drift).is_err());
}

#[test]
fn imputation_metrics_smoke_schema_matches_model_authority() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let path = repository_root
        .join("benchmarks/contracts/schemas/local-vcf-imputation-metrics-smoke.schema.json");
    let generated = serde_json::to_value(benchmark_local_vcf_imputation_metrics_smoke_schema())?;
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(checked, generated, "imputation-metrics smoke schema drifted from its model owner");
    Ok(())
}

#[test]
fn imputation_metrics_smoke_record_rejects_unknown_fields() -> anyhow::Result<()> {
    let mut value = serde_json::to_value(report())?;
    value["legacy_status"] = serde_json::json!("passed");
    assert!(serde_json::from_value::<BenchmarkLocalVcfImputationMetricsSmokeReport>(value).is_err());
    Ok(())
}
