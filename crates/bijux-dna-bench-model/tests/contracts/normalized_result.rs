use bijux_dna_bench_model::normalized_result::bam::{
    bam_coverage_result_schema, bam_duplication_metrics_result_schema,
    bam_endogenous_content_result_schema, bam_genotyping_result_schema,
    bam_mapping_summary_result_schema, BenchmarkBamCoverageResult,
    BenchmarkBamDuplicationMetricsResult, BenchmarkBamEndogenousContentResult,
    BenchmarkBamGenotypingResult, BenchmarkBamMappingSummaryResult,
};

#[test]
fn normalized_bam_results_reject_unknown_fields_and_invalid_tools() -> anyhow::Result<()> {
    let cases = [
        serde_json::json!({
            "schema_version": "bijux.benchmark.bam.coverage.bundle.v1",
            "stage_id": "bam.coverage",
            "tool_id": "samtools",
            "sample_id": "ERR2112767",
            "reference_bases": 10,
            "covered_bases": 6,
            "coverage_fraction": 0.6,
            "mean_depth": 1.2,
        }),
        serde_json::json!({
            "schema_version": "bijux.benchmark.bam.duplication_metrics.bundle.v1",
            "stage_id": "bam.duplication_metrics",
            "tool_id": "picard",
            "sample_id": "ERR2112767",
            "examined_reads": 10,
            "duplicate_reads": 2,
            "duplication_fraction": 0.2,
        }),
        serde_json::json!({
            "schema_version": "bijux.benchmark.bam.endogenous_content.bundle.v1",
            "stage_id": "bam.endogenous_content",
            "tool_id": "samtools",
            "sample_id": "ERR2112767",
            "total_primary_reads": 10,
            "mapped_primary_reads": 8,
            "mapped_fraction": 0.8,
            "mapped_nonduplicate_primary_reads": 7,
        }),
        serde_json::json!({
            "schema_version": "bijux.benchmark.bam.genotyping.bundle.v1",
            "stage_id": "bam.genotyping",
            "tool_id": "bcftools",
            "sample_id": "ERR2112767",
            "total_records": 10,
            "covered_sites": 8,
            "called_genotypes": 7,
            "alternate_genotypes": 2,
            "mean_depth_at_covered_sites": 1.2,
        }),
        serde_json::json!({
            "schema_version": "bijux.benchmark.bam.mapping_summary.bundle.v1",
            "stage_id": "bam.mapping_summary",
            "tool_id": "samtools",
            "sample_id": "ERR2112767",
            "primary_reads": 10,
            "mapped_primary_reads": 8,
            "mapped_primary_fraction": 0.8,
            "paired_primary_reads": 10,
        }),
    ];
    serde_json::from_value::<BenchmarkBamCoverageResult>(cases[0].clone())?;
    serde_json::from_value::<BenchmarkBamDuplicationMetricsResult>(cases[1].clone())?;
    serde_json::from_value::<BenchmarkBamEndogenousContentResult>(cases[2].clone())?;
    serde_json::from_value::<BenchmarkBamGenotypingResult>(cases[3].clone())?;
    serde_json::from_value::<BenchmarkBamMappingSummaryResult>(cases[4].clone())?;

    let mut unknown = cases[0].clone();
    unknown["invented"] = serde_json::json!(true);
    assert!(serde_json::from_value::<BenchmarkBamCoverageResult>(unknown).is_err());

    let mut invalid_tool = cases[2].clone();
    invalid_tool["tool_id"] = serde_json::json!("picard");
    assert!(serde_json::from_value::<BenchmarkBamEndogenousContentResult>(invalid_tool).is_err());
    Ok(())
}

#[test]
fn checked_normalized_bam_schemas_match_model_authority() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let schemas = [
        ("bam-coverage-result.schema.json", serde_json::to_value(bam_coverage_result_schema())?),
        (
            "bam-duplication-metrics-result.schema.json",
            serde_json::to_value(bam_duplication_metrics_result_schema())?,
        ),
        (
            "bam-endogenous-content-result.schema.json",
            serde_json::to_value(bam_endogenous_content_result_schema())?,
        ),
        (
            "bam-genotyping-result.schema.json",
            serde_json::to_value(bam_genotyping_result_schema())?,
        ),
        (
            "bam-mapping-summary-result.schema.json",
            serde_json::to_value(bam_mapping_summary_result_schema())?,
        ),
    ];
    let schema_root = repository_root.join("benchmarks/contracts/schemas");
    for (name, generated) in schemas {
        let path = schema_root.join(name);
        if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
            let mut bytes = serde_json::to_vec_pretty(&generated)?;
            bytes.push(b'\n');
            std::fs::write(&path, bytes)?;
        }
        let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
        assert_eq!(checked, generated, "normalized BAM schema drifted: {name}");
    }
    Ok(())
}
