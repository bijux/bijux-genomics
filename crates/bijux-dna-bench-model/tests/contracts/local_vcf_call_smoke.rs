use std::collections::BTreeMap;

use bijux_dna_bench_model::{
    benchmark_local_vcf_call_smoke_metrics_schema, benchmark_local_vcf_call_smoke_schema,
    validate_benchmark_local_vcf_call_smoke_metrics,
    validate_benchmark_local_vcf_call_smoke_report, BenchmarkLocalVcfCallSmokeMetrics,
    BenchmarkLocalVcfCallSmokeReport, BENCHMARK_LOCAL_VCF_CALL_SMOKE_METRICS_SCHEMA_V1,
    BENCHMARK_LOCAL_VCF_CALL_SMOKE_SCHEMA_V1,
};

fn metrics() -> BenchmarkLocalVcfCallSmokeMetrics {
    BenchmarkLocalVcfCallSmokeMetrics {
        schema_version: BENCHMARK_LOCAL_VCF_CALL_SMOKE_METRICS_SCHEMA_V1.to_string(),
        stage_id: "vcf.call".to_string(),
        resolved_stage_id: "vcf.call_diploid".to_string(),
        corpus_id: "vcf_production_regression".to_string(),
        sample_id: "human_like_validation".to_string(),
        input_bam: "corpus/aligned.bam".to_string(),
        reference: "corpus/reference.fasta".to_string(),
        variant_count: 3,
        snp_count: 2,
        indel_count: 1,
        sample_count: 1,
        tool_id: "bcftools".to_string(),
        exit_code: 0,
    }
}

fn report() -> BenchmarkLocalVcfCallSmokeReport {
    BenchmarkLocalVcfCallSmokeReport {
        schema_version: BENCHMARK_LOCAL_VCF_CALL_SMOKE_SCHEMA_V1.to_string(),
        command: "bijux-dna bench local run-vcf-call-smoke --tool-id bcftools".to_string(),
        stage_id: "vcf.call".to_string(),
        resolved_stage_id: "vcf.call_diploid".to_string(),
        tool_id: "bcftools".to_string(),
        corpus_id: "vcf_production_regression".to_string(),
        sample_id: "human_like_validation".to_string(),
        sample_name: "core-v1-pass".to_string(),
        input_bam: "corpus/aligned.bam".to_string(),
        input_bam_index: "corpus/aligned.bam.bai".to_string(),
        reference: "corpus/reference.fasta".to_string(),
        materialized_reference_path: "runs/reference.fasta".to_string(),
        output_root: "runs/vcf.call/bcftools".to_string(),
        output_vcf_path: "runs/vcf.call/bcftools/calls.vcf.gz".to_string(),
        output_tbi_path: "runs/vcf.call/bcftools/calls.vcf.gz.tbi".to_string(),
        metrics_path: "runs/vcf.call/bcftools/metrics.json".to_string(),
        stage_result_manifest_path: "runs/vcf.call/bcftools/stage-result.json".to_string(),
        started_at: "unix:1".to_string(),
        finished_at: "unix:2".to_string(),
        elapsed_seconds: 0.5,
        exit_code: 0,
        variant_count: 3,
        snp_count: 2,
        indel_count: 1,
        sample_count: 1,
        parseable: true,
        validation_checks: BTreeMap::from([
            ("bgzip".to_string(), true),
            ("contig_header_sane".to_string(), true),
            ("sorted".to_string(), true),
            ("tabix_index".to_string(), true),
        ]),
        gt_present: true,
        gl_present: false,
    }
}

#[test]
fn local_vcf_call_smoke_contract_accepts_complete_metrics_and_report() -> anyhow::Result<()> {
    validate_benchmark_local_vcf_call_smoke_metrics(&metrics())?;
    validate_benchmark_local_vcf_call_smoke_report(&report())?;
    Ok(())
}

#[test]
fn local_vcf_call_smoke_contract_rejects_count_and_validation_drift() {
    let mut count_drift = metrics();
    count_drift.variant_count = 4;
    assert!(validate_benchmark_local_vcf_call_smoke_metrics(&count_drift).is_err());

    let mut validation_drift = report();
    validation_drift.validation_checks.insert("sorted".to_string(), false);
    assert!(validate_benchmark_local_vcf_call_smoke_report(&validation_drift).is_err());

    let mut field_drift = report();
    field_drift.gt_present = false;
    assert!(validate_benchmark_local_vcf_call_smoke_report(&field_drift).is_err());
}

#[test]
fn local_vcf_call_smoke_schemas_match_model_authority() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    for (relative_path, generated) in [
        (
            "benchmarks/contracts/schemas/local-vcf-call-smoke.schema.json",
            serde_json::to_value(benchmark_local_vcf_call_smoke_schema())?,
        ),
        (
            "benchmarks/contracts/schemas/local-vcf-call-smoke-metrics.schema.json",
            serde_json::to_value(benchmark_local_vcf_call_smoke_metrics_schema())?,
        ),
    ] {
        let path = repository_root.join(relative_path);
        if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
            let mut bytes = serde_json::to_vec_pretty(&generated)?;
            bytes.push(b'\n');
            std::fs::write(&path, bytes)?;
        }
        let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
        assert_eq!(checked, generated, "{relative_path} drifted from its model owner");
    }
    Ok(())
}

#[test]
fn local_vcf_call_smoke_records_reject_unknown_fields() -> anyhow::Result<()> {
    let mut metrics_value = serde_json::to_value(metrics())?;
    metrics_value["legacy_status"] = serde_json::json!("passed");
    assert!(serde_json::from_value::<BenchmarkLocalVcfCallSmokeMetrics>(metrics_value).is_err());

    let mut report_value = serde_json::to_value(report())?;
    report_value["legacy_status"] = serde_json::json!("passed");
    assert!(serde_json::from_value::<BenchmarkLocalVcfCallSmokeReport>(report_value).is_err());
    Ok(())
}
