use bijux_dna_bench_model::normalized_result::bam::{
    bam_align_cohort_metrics_schema, bam_align_metrics_schema, bam_align_result_schema,
    bam_coverage_metrics_schema, bam_coverage_result_schema, bam_damage_metrics_schema,
    bam_damage_result_schema, bam_duplication_metrics_measurements_schema,
    bam_duplication_metrics_result_schema, bam_endogenous_content_metrics_schema,
    bam_endogenous_content_result_schema, bam_gc_bias_cohort_metrics_schema,
    bam_gc_bias_metrics_schema, bam_gc_bias_result_schema, bam_genotyping_metrics_schema,
    bam_genotyping_result_schema, bam_insert_size_cohort_metrics_schema,
    bam_insert_size_metrics_schema, bam_insert_size_result_schema,
    bam_mapping_summary_metrics_schema, bam_mapping_summary_result_schema,
    BenchmarkBamAlignCohortMetrics, BenchmarkBamAlignMetrics, BenchmarkBamAlignResult,
    BenchmarkBamCoverageMetrics, BenchmarkBamCoverageResult, BenchmarkBamDamageMetrics,
    BenchmarkBamDamageResult, BenchmarkBamDuplicationMetricsResult,
    BenchmarkBamEndogenousContentResult, BenchmarkBamGcBiasCohortMetrics,
    BenchmarkBamGcBiasMetrics, BenchmarkBamGcBiasResult, BenchmarkBamGenotypingResult,
    BenchmarkBamInsertSizeCohortMetrics, BenchmarkBamInsertSizeMetrics,
    BenchmarkBamInsertSizeResult, BenchmarkBamMappingSummaryResult,
};
use bijux_dna_bench_model::normalized_result::bam_envelope::{
    bam_normalized_metrics_stage_contracts, render_bam_normalized_metrics_schema,
    validate_bam_normalized_metrics, BAM_NORMALIZED_METRICS_SCHEMA_VERSION,
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
use bijux_dna_bench_model::normalized_result::fastq_envelope::{
    fastq_normalized_metrics_stage_contracts, render_fastq_normalized_metrics_schema,
    validate_fastq_normalized_metrics, FASTQ_NORMALIZED_METRICS_SCHEMA_VERSION,
};
use bijux_dna_bench_model::normalized_result::vcf_envelope::{
    render_vcf_normalized_metrics_schema, render_vcf_normalized_metrics_stage_schema,
    validate_vcf_normalized_metrics, vcf_normalized_metrics_stage_descriptors,
};
use bijux_dna_bench_model::normalized_result::{
    bam_downstream_probe_schema, BenchmarkDownstreamProbeRecord,
};

#[test]
fn bam_execution_metrics_envelope_is_model_owned_and_matches_checked_schema() -> anyhow::Result<()>
{
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let checked_path = repository_root.join("benchmarks/schemas/bam-normalized-metrics.v1.json");
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(checked_path)?)?;
    assert_eq!(checked, render_bam_normalized_metrics_schema());

    let contracts = bam_normalized_metrics_stage_contracts();
    assert_eq!(contracts.len(), 24);
    let unique_stage_ids = contracts
        .iter()
        .map(|contract| contract.stage_id)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(unique_stage_ids.len(), contracts.len());

    let payload = serde_json::json!({
        "schema_version": BAM_NORMALIZED_METRICS_SCHEMA_VERSION,
        "stage_id": "bam.damage",
        "tool_id": "mapdamage2",
        "tool_version": "2.2.3",
        "execution": {
            "runtime_s": 1.25,
            "memory_mb": 128.0,
            "exit_code": 0,
            "status": "ok"
        },
        "outputs_count": 1,
        "artifacts": {
            "required_count": 1,
            "missing_required": [],
            "items": {
                "damage_profile": { "path": "results/mapdamage", "exists": true }
            }
        },
        "contracts": {
            "bam_invariants": true,
            "output_contract": true,
            "tool_wrapper": true
        },
        "normalized_keys": [
            "stage_id",
            "tool_id",
            "tool_version",
            "execution.runtime_s",
            "execution.memory_mb",
            "execution.exit_code",
            "artifacts.required_count",
            "artifacts.missing_required",
            "contracts.bam_invariants",
            "contracts.output_contract",
            "contracts.tool_wrapper"
        ]
    });
    assert_eq!(validate_bam_normalized_metrics(&payload)?.stage_id, "bam.damage");

    let mut unknown_stage = payload;
    unknown_stage["stage_id"] = serde_json::json!("bam.unknown");
    assert!(validate_bam_normalized_metrics(&unknown_stage).is_err());
    Ok(())
}

#[test]
fn fastq_execution_metrics_envelope_is_model_owned_and_matches_checked_schema() -> anyhow::Result<()>
{
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let checked_path = repository_root.join("benchmarks/schemas/fastq-normalized-metrics.v1.json");
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(checked_path)?)?;
    assert_eq!(checked, render_fastq_normalized_metrics_schema());

    let contracts = fastq_normalized_metrics_stage_contracts();
    assert_eq!(contracts.len(), 27);
    let unique_stage_ids = contracts
        .iter()
        .map(|contract| contract.stage_id)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(unique_stage_ids.len(), contracts.len());

    let payload = serde_json::json!({
        "schema_version": FASTQ_NORMALIZED_METRICS_SCHEMA_VERSION,
        "stage": "fastq.validate_reads",
        "report_json": "validate-reads.json",
        "validator": "seqkit",
        "failure_class": null,
        "strict_pass": true,
        "exit_code": 0
    });
    assert_eq!(validate_fastq_normalized_metrics(&payload)?.stage_id, "fastq.validate_reads");

    let mut unknown_stage = payload;
    unknown_stage["stage"] = serde_json::json!("fastq.unknown");
    assert!(validate_fastq_normalized_metrics(&unknown_stage).is_err());
    Ok(())
}

#[test]
fn vcf_execution_metrics_envelope_is_model_owned_and_matches_checked_schemas() -> anyhow::Result<()>
{
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(
        repository_root.join("benchmarks/schemas/vcf-normalized-metrics.v1.json"),
    )?)?;
    assert_eq!(checked, render_vcf_normalized_metrics_schema()?);

    let descriptors = vcf_normalized_metrics_stage_descriptors()?;
    assert_eq!(descriptors.len(), 20);
    let unique_stage_ids = descriptors
        .iter()
        .map(|descriptor| descriptor.stage_id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(unique_stage_ids.len(), descriptors.len());
    for descriptor in descriptors {
        let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(
            repository_root
                .join("benchmarks/schemas/vcf-normalized-metrics")
                .join(&descriptor.file_name),
        )?)?;
        assert_eq!(checked, render_vcf_normalized_metrics_stage_schema(&descriptor.stage_id)?);
    }

    let payload = serde_json::json!({
        "schema_version": "bijux.vcf.call.v1",
        "stage_id": "vcf.call",
        "tool_id": "bcftools",
        "variant_count": 4,
        "snp_count": 3,
        "indel_count": 1,
        "sample_count": 2
    });
    assert_eq!(validate_vcf_normalized_metrics(&payload)?.stage_id, "vcf.call");

    let mut unknown_stage = payload;
    unknown_stage["stage_id"] = serde_json::json!("vcf.unknown");
    assert!(validate_vcf_normalized_metrics(&unknown_stage).is_err());
    Ok(())
}

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
fn normalized_bam_gc_bias_result_rejects_invalid_tool() -> anyhow::Result<()> {
    let value = serde_json::json!({
        "schema_version": "bijux.benchmark.bam.gc_bias.bundle.v1",
        "stage_id": "bam.gc_bias",
        "tool_id": "picard",
        "sample_id": "ERR2112767",
        "gc_bias_score": 0.054,
        "at_dropout": 3.2,
        "gc_dropout": 5.4,
        "total_clusters": 200_000,
        "aligned_reads": 180_000,
        "windows": 1200,
        "read_starts": 180_000,
    });
    serde_json::from_value::<BenchmarkBamGcBiasResult>(value.clone())?;
    let mut invalid_tool = value;
    invalid_tool["tool_id"] = serde_json::json!("samtools");
    assert!(serde_json::from_value::<BenchmarkBamGcBiasResult>(invalid_tool).is_err());
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
        (
            "bam-align-cohort-metrics.schema.json",
            serde_json::to_value(bam_align_cohort_metrics_schema())?,
        ),
        ("bam-coverage-result.schema.json", serde_json::to_value(bam_coverage_result_schema())?),
        ("bam-coverage-metrics.schema.json", serde_json::to_value(bam_coverage_metrics_schema())?),
        (
            "bam-duplication-metrics-result.schema.json",
            serde_json::to_value(bam_duplication_metrics_result_schema())?,
        ),
        ("bam-damage-result.schema.json", serde_json::to_value(bam_damage_result_schema())?),
        ("bam-damage-metrics.schema.json", serde_json::to_value(bam_damage_metrics_schema())?),
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
        (
            "bam-insert-size-cohort-metrics.schema.json",
            serde_json::to_value(bam_insert_size_cohort_metrics_schema())?,
        ),
        ("bam-gc-bias-result.schema.json", serde_json::to_value(bam_gc_bias_result_schema())?),
        ("bam-gc-bias-metrics.schema.json", serde_json::to_value(bam_gc_bias_metrics_schema())?),
        (
            "bam-gc-bias-cohort-metrics.schema.json",
            serde_json::to_value(bam_gc_bias_cohort_metrics_schema())?,
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
fn normalized_bam_alignment_and_coverage_metrics_fail_closed() -> anyhow::Result<()> {
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
    Ok(())
}

#[test]
fn normalized_bam_damage_technical_measurements_and_probes_fail_closed() -> anyhow::Result<()> {
    let damage_result = serde_json::json!({
        "schema_version": "bijux.benchmark.bam.damage.bundle.v1",
        "stage_id": "bam.damage",
        "tool_id": "ngsbriggs",
        "sample_id": "ERR2112767",
        "normalization_complete": 1,
        "native_file_count": 2,
        "native_output_bytes": 128,
        "authenticity_certified": false
    });
    serde_json::from_value::<BenchmarkBamDamageResult>(damage_result.clone())?;
    let mut unsupported_damage = damage_result;
    unsupported_damage["authenticity_certified"] = serde_json::json!(true);
    assert!(serde_json::from_value::<BenchmarkBamDamageResult>(unsupported_damage).is_err());

    serde_json::from_value::<BenchmarkBamDamageMetrics>(serde_json::json!({
        "schema_version": "bijux.benchmark.bam.damage.metrics.v1",
        "normalization_complete": 1,
        "native_file_count": 2,
        "native_output_bytes": 128,
        "bam_input_bytes": 100,
        "bam_index_input_bytes": 10,
        "reference_input_bytes": 80,
        "input_bytes": 190,
        "output_bytes": 128
    }))?;

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

    let gc_bias_metrics = serde_json::json!({
        "schema_version": "bijux.benchmark.bam.gc_bias.metrics.v1",
        "gc_bias_score": 0.054,
        "at_dropout": 3.2,
        "gc_dropout": 5.4,
        "total_clusters": 200_000,
        "aligned_reads": 180_000,
        "windows": 1200,
        "read_starts": 180_000,
        "bam_input_bytes": 100,
        "bam_index_input_bytes": 10,
        "reference_input_bytes": 80,
        "input_bytes": 190,
        "output_bytes": 20
    });
    serde_json::from_value::<BenchmarkBamGcBiasMetrics>(gc_bias_metrics)?;

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

    let damage_probe = serde_json::json!({
        "schema_version": "bijux.benchmark.downstream_probe.v1",
        "stage_id": "benchmark.bam.damage.consume",
        "input_contract": "bijux.benchmark.bam.damage.bundle.v1",
        "complete": true,
        "authenticity_certified": false
    });
    serde_json::from_value::<BenchmarkDownstreamProbeRecord>(damage_probe.clone())?;
    let mut invalid_damage_probe = damage_probe;
    invalid_damage_probe["authenticity_certified"] = serde_json::json!(true);
    assert!(serde_json::from_value::<BenchmarkDownstreamProbeRecord>(invalid_damage_probe).is_err());
    Ok(())
}

#[test]
fn normalized_bam_cohort_metrics_fail_closed() -> anyhow::Result<()> {
    let align = serde_json::json!({
        "sample_count": 10,
        "total_reads": 100.0,
        "mapped_reads": 80.0,
        "alignment_rate": 0.8,
        "expected_mapped_reads": 90.0,
        "exact_placement_reads": 70.0,
        "placement_precision": 0.875,
        "placement_recall": 0.777_777_777_778,
        "input_bytes": 210,
        "output_bytes": 50
    });
    serde_json::from_value::<BenchmarkBamAlignCohortMetrics>(align.clone())?;
    let mut unknown = align;
    unknown["invented"] = serde_json::json!(true);
    assert!(serde_json::from_value::<BenchmarkBamAlignCohortMetrics>(unknown).is_err());

    serde_json::from_value::<BenchmarkBamGcBiasCohortMetrics>(serde_json::json!({
        "sample_count": 10,
        "gc_bias_score": 0.2,
        "at_dropout": 1.0,
        "gc_dropout": 20.0,
        "total_clusters": 100.0,
        "aligned_reads": 80.0,
        "windows": 50.0,
        "read_starts": 70.0,
        "input_bytes": 210,
        "output_bytes": 50
    }))?;
    serde_json::from_value::<BenchmarkBamInsertSizeCohortMetrics>(serde_json::json!({
        "sample_count": 10,
        "median_insert_size": 100.0,
        "mean_insert_size": 110.0,
        "standard_deviation": 20.0,
        "median_absolute_deviation": 15.0,
        "minimum_insert_size": 50.0,
        "maximum_insert_size": 200.0,
        "read_pairs": 100.0,
        "pair_orientation_fr_fraction": 1.0,
        "input_bytes": 210,
        "output_bytes": 50
    }))?;
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
