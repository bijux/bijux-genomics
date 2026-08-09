use bijux_dna_bench_model::normalized_result::bam::{
    bam_align_metrics_schema, bam_align_result_schema, bam_coverage_metrics_schema,
    bam_coverage_result_schema, bam_duplication_metrics_measurements_schema,
    bam_duplication_metrics_result_schema, bam_endogenous_content_metrics_schema,
    bam_endogenous_content_result_schema, bam_genotyping_metrics_schema,
    bam_genotyping_result_schema, bam_insert_size_metrics_schema, bam_insert_size_result_schema,
    bam_mapping_summary_metrics_schema, bam_mapping_summary_result_schema,
    BenchmarkBamAlignMetrics, BenchmarkBamAlignResult, BenchmarkBamCoverageMetrics,
    BenchmarkBamCoverageResult, BenchmarkBamDuplicationMetricsResult,
    BenchmarkBamEndogenousContentResult, BenchmarkBamGenotypingResult,
    BenchmarkBamInsertSizeMetrics, BenchmarkBamInsertSizeResult, BenchmarkBamMappingSummaryResult,
};
use bijux_dna_bench_model::normalized_result::fastq::{
    fastq_detect_adapters_metrics_schema, fastq_detect_adapters_probe_schema,
    fastq_detect_adapters_result_schema, fastq_profile_read_lengths_metrics_schema,
    fastq_profile_read_lengths_probe_schema, fastq_profile_read_lengths_result_schema,
    fastq_profile_reads_metrics_schema, fastq_profile_reads_probe_schema,
    fastq_profile_reads_result_schema, fastq_validate_reads_metrics_schema,
    fastq_validate_reads_probe_schema, fastq_validate_reads_result_schema,
    BenchmarkFastqProfileMetrics, BenchmarkFastqProfileProbe, BenchmarkFastqProfileResult,
};
use bijux_dna_bench_model::normalized_result::{
    bam_downstream_probe_schema, BenchmarkDownstreamProbeRecord,
};

#[test]
fn normalized_bam_results_reject_unknown_fields_and_invalid_tools() -> anyhow::Result<()> {
    let cases = [
        serde_json::json!({
            "schema_version": "bijux.benchmark.bam.align.bundle.v1",
            "stage_id": "bam.align",
            "tool_id": "bwa",
            "sample_id": "ERR2112767",
            "total_reads": 10,
            "mapped_reads": 8,
            "alignment_rate": 0.8,
            "expected_mapped_reads": 9,
            "exact_placement_reads": 7,
            "placement_precision": 0.875,
            "placement_recall": 0.777_777_777_778,
        }),
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
        serde_json::json!({
            "schema_version": "bijux.benchmark.bam.insert_size.bundle.v1",
            "stage_id": "bam.insert_size",
            "tool_id": "picard",
            "sample_id": "ERR2112767",
            "median_insert_size": 62.0,
            "mean_insert_size": 64.2,
            "standard_deviation": 12.4,
            "median_absolute_deviation": 8.0,
            "minimum_insert_size": 30,
            "maximum_insert_size": 180,
            "read_pairs": 1000,
            "pair_orientation_fr_fraction": 1.0,
        }),
    ];
    serde_json::from_value::<BenchmarkBamAlignResult>(cases[0].clone())?;
    serde_json::from_value::<BenchmarkBamCoverageResult>(cases[1].clone())?;
    serde_json::from_value::<BenchmarkBamDuplicationMetricsResult>(cases[2].clone())?;
    serde_json::from_value::<BenchmarkBamEndogenousContentResult>(cases[3].clone())?;
    serde_json::from_value::<BenchmarkBamGenotypingResult>(cases[4].clone())?;
    serde_json::from_value::<BenchmarkBamMappingSummaryResult>(cases[5].clone())?;
    serde_json::from_value::<BenchmarkBamInsertSizeResult>(cases[6].clone())?;

    let mut unknown = cases[1].clone();
    unknown["invented"] = serde_json::json!(true);
    assert!(serde_json::from_value::<BenchmarkBamCoverageResult>(unknown).is_err());

    let mut invalid_tool = cases[3].clone();
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
        ("bam-align-result.schema.json", serde_json::to_value(bam_align_result_schema())?),
        ("bam-align-metrics.schema.json", serde_json::to_value(bam_align_metrics_schema())?),
        ("bam-coverage-result.schema.json", serde_json::to_value(bam_coverage_result_schema())?),
        ("bam-coverage-metrics.schema.json", serde_json::to_value(bam_coverage_metrics_schema())?),
        (
            "bam-duplication-metrics-result.schema.json",
            serde_json::to_value(bam_duplication_metrics_result_schema())?,
        ),
        (
            "bam-duplication-metrics-measurements.schema.json",
            serde_json::to_value(bam_duplication_metrics_measurements_schema())?,
        ),
        (
            "bam-endogenous-content-result.schema.json",
            serde_json::to_value(bam_endogenous_content_result_schema())?,
        ),
        (
            "bam-endogenous-content-metrics.schema.json",
            serde_json::to_value(bam_endogenous_content_metrics_schema())?,
        ),
        (
            "bam-genotyping-result.schema.json",
            serde_json::to_value(bam_genotyping_result_schema())?,
        ),
        (
            "bam-genotyping-metrics.schema.json",
            serde_json::to_value(bam_genotyping_metrics_schema())?,
        ),
        (
            "bam-mapping-summary-result.schema.json",
            serde_json::to_value(bam_mapping_summary_result_schema())?,
        ),
        (
            "bam-mapping-summary-metrics.schema.json",
            serde_json::to_value(bam_mapping_summary_metrics_schema())?,
        ),
        (
            "bam-insert-size-result.schema.json",
            serde_json::to_value(bam_insert_size_result_schema())?,
        ),
        (
            "bam-insert-size-metrics.schema.json",
            serde_json::to_value(bam_insert_size_metrics_schema())?,
        ),
        ("bam-downstream-probe.schema.json", serde_json::to_value(bam_downstream_probe_schema())?),
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

#[test]
fn normalized_bam_measurements_and_probes_fail_closed() -> anyhow::Result<()> {
    let align_metrics = serde_json::json!({
        "schema_version": "bijux.benchmark.bam.align.metrics.v1",
        "total_reads": 10,
        "mapped_reads": 8,
        "alignment_rate": 0.8,
        "expected_mapped_reads": 9,
        "exact_placement_reads": 7,
        "placement_precision": 0.875,
        "placement_recall": 0.777_777_777_778,
        "fastq_input_bytes": 100,
        "reference_input_bytes": 80,
        "reference_index_input_bytes": 20,
        "truth_input_bytes": 10,
        "input_bytes": 210,
        "output_bytes": 50
    });
    serde_json::from_value::<BenchmarkBamAlignMetrics>(align_metrics)?;

    let metrics = serde_json::json!({
        "schema_version": "bijux.benchmark.bam.coverage.metrics.v1",
        "reference_bases": 10,
        "covered_bases": 6,
        "coverage_fraction": 0.6,
        "mean_depth": 1.2,
        "bam_input_bytes": 100,
        "bam_index_input_bytes": 10,
        "input_bytes": 110,
        "output_bytes": 20
    });
    serde_json::from_value::<BenchmarkBamCoverageMetrics>(metrics.clone())?;
    let mut unsupported_metrics = metrics;
    unsupported_metrics["schema_version"] =
        serde_json::json!("bijux.benchmark.bam.coverage.metrics.v2");
    assert!(serde_json::from_value::<BenchmarkBamCoverageMetrics>(unsupported_metrics).is_err());

    let insert_size_metrics = serde_json::json!({
        "schema_version": "bijux.benchmark.bam.insert_size.metrics.v1",
        "median_insert_size": 62.0,
        "mean_insert_size": 64.2,
        "standard_deviation": 12.4,
        "median_absolute_deviation": 8.0,
        "minimum_insert_size": 30,
        "maximum_insert_size": 180,
        "read_pairs": 1000,
        "pair_orientation_fr_fraction": 1.0,
        "bam_input_bytes": 100,
        "bam_index_input_bytes": 10,
        "input_bytes": 110,
        "output_bytes": 20
    });
    serde_json::from_value::<BenchmarkBamInsertSizeMetrics>(insert_size_metrics)?;

    let probe = serde_json::json!({
        "schema_version": "bijux.benchmark.downstream_probe.v1",
        "stage_id": "benchmark.bam.coverage.compare",
        "input_contract": "bijux.benchmark.bam.coverage.bundle.v1",
        "complete": true
    });
    serde_json::from_value::<BenchmarkDownstreamProbeRecord>(probe.clone())?;
    let mut unsupported_probe = probe;
    unsupported_probe["stage_id"] = serde_json::json!("benchmark.bam.invented.consume");
    assert!(serde_json::from_value::<BenchmarkDownstreamProbeRecord>(unsupported_probe).is_err());
    Ok(())
}

#[test]
fn normalized_fastq_results_reject_unknown_fields() -> anyhow::Result<()> {
    let mut value = serde_json::json!({
        "schema_version": "bijux.benchmark.fastq.profile_reads.bundle.v1",
        "stage_id": "fastq.profile_reads",
        "tool_id": "seqkit",
        "results": [{
            "sample_id": "ERR2112767",
            "input_layout": "single_end",
            "file_count": 1,
            "read_count": 10,
            "base_count": 100,
            "minimum_read_length": 10,
            "maximum_read_length": 10
        }]
    });
    serde_json::from_value::<BenchmarkFastqProfileResult>(value.clone())?;
    value["results"][0]["invented"] = serde_json::json!(true);
    assert!(serde_json::from_value::<BenchmarkFastqProfileResult>(value).is_err());
    Ok(())
}

#[test]
fn normalized_fastq_evidence_rejects_unknown_identities() -> anyhow::Result<()> {
    let metrics = serde_json::json!({
        "schema_version": "bijux.benchmark.fastq.profile_reads.metrics.v1",
        "sample_count": 1,
        "file_count": 1,
        "read_count": 10,
        "base_count": 100,
        "minimum_read_length": 10,
        "maximum_read_length": 10,
        "admission_mismatch_count": 0,
        "input_bytes": 100,
        "output_bytes": 20
    });
    serde_json::from_value::<BenchmarkFastqProfileMetrics>(metrics.clone())?;
    let mut unsupported_metrics = metrics;
    unsupported_metrics["schema_version"] =
        serde_json::json!("bijux.benchmark.fastq.profile_reads.metrics.v2");
    assert!(serde_json::from_value::<BenchmarkFastqProfileMetrics>(unsupported_metrics).is_err());

    let probe = serde_json::json!({
        "schema_version": "bijux.benchmark.fastq.report_qc.probe.v1",
        "consumer_stage_id": "benchmark.fastq.profile_reads.compare",
        "input_contract": "bijux.benchmark.fastq.profile_reads.bundle.v1",
        "sample_count": 1,
        "file_count": 1,
        "read_count": 10,
        "base_count": 100,
        "minimum_read_length": 10,
        "maximum_read_length": 10
    });
    serde_json::from_value::<BenchmarkFastqProfileProbe>(probe.clone())?;
    let mut unsupported_probe = probe;
    unsupported_probe["consumer_stage_id"] = serde_json::json!("benchmark.fastq.invented.compare");
    assert!(serde_json::from_value::<BenchmarkFastqProfileProbe>(unsupported_probe).is_err());
    Ok(())
}

#[test]
fn checked_normalized_fastq_schemas_match_model_authority() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let schemas = [
        (
            "fastq-detect-adapters-result.schema.json",
            serde_json::to_value(fastq_detect_adapters_result_schema())?,
        ),
        (
            "fastq-detect-adapters-metrics.schema.json",
            serde_json::to_value(fastq_detect_adapters_metrics_schema())?,
        ),
        (
            "fastq-detect-adapters-probe.schema.json",
            serde_json::to_value(fastq_detect_adapters_probe_schema())?,
        ),
        (
            "fastq-profile-read-lengths-result.schema.json",
            serde_json::to_value(fastq_profile_read_lengths_result_schema())?,
        ),
        (
            "fastq-profile-read-lengths-metrics.schema.json",
            serde_json::to_value(fastq_profile_read_lengths_metrics_schema())?,
        ),
        (
            "fastq-profile-read-lengths-probe.schema.json",
            serde_json::to_value(fastq_profile_read_lengths_probe_schema())?,
        ),
        (
            "fastq-profile-reads-result.schema.json",
            serde_json::to_value(fastq_profile_reads_result_schema())?,
        ),
        (
            "fastq-profile-reads-metrics.schema.json",
            serde_json::to_value(fastq_profile_reads_metrics_schema())?,
        ),
        (
            "fastq-profile-reads-probe.schema.json",
            serde_json::to_value(fastq_profile_reads_probe_schema())?,
        ),
        (
            "fastq-validate-reads-result.schema.json",
            serde_json::to_value(fastq_validate_reads_result_schema())?,
        ),
        (
            "fastq-validate-reads-metrics.schema.json",
            serde_json::to_value(fastq_validate_reads_metrics_schema())?,
        ),
        (
            "fastq-validate-reads-probe.schema.json",
            serde_json::to_value(fastq_validate_reads_probe_schema())?,
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
        assert_eq!(checked, generated, "normalized FASTQ schema drifted: {name}");
    }
    Ok(())
}
