use bijux_dna_analyze::{
    metric_set, BenchmarkRecord, FastqCorrectMetrics, FastqDeltaMetrics, FastqDepleteHostMetrics,
    FastqDepleteReferenceContaminantsMetrics, FastqDuplicateMetrics, FastqFilterMetrics,
    FastqIndexReferenceMetrics, FastqLowComplexityMetrics, FastqMergeMetrics, FastqQcPostMetrics,
    FastqScreenMetrics, FastqStatsMetrics, FastqTrimMetrics, FastqUmiMetrics, FastqValidateMetrics,
    LengthHistogramBin, MetricSet,
};
use bijux_dna_core::prelude::measure::ExecutionMetrics;

fn base_record(metrics: MetricSet<FastqTrimMetrics>) -> BenchmarkRecord<FastqTrimMetrics> {
    BenchmarkRecord {
        context: bijux_dna_analyze::BenchmarkContext {
            tool: "fastp".to_string(),
            tool_version: "0.23.4".to_string(),
            image_digest: "sha256:abc".to_string(),
            runner: "docker".to_string(),
            platform: "local".to_string(),
            input_hash: "sha256:deadbeef".to_string(),
            parameters: bijux_dna_analyze::model::JsonBlob::from_pairs(&[("sample_id", "s1")]),
        },
        execution: ExecutionMetrics {
            runtime_s: 1.0,
            memory_mb: 32.0,
            exit_code: 0,
        },
        metrics,
    }
}

#[test]
fn metrics_schema_matches_stage_and_version() {
    let record = base_record(metric_set(FastqTrimMetrics {
        reads_in: 100,
        reads_out: 90,
        bases_in: 1000,
        bases_out: 900,
        pairs_in: None,
        pairs_out: None,
        mean_q_before: 30.0,
        mean_q_after: 31.0,
        delta_metrics: FastqDeltaMetrics {
            read_retention: 0.9,
            base_retention: 0.9,
            mean_q_delta: 1.0,
            gc_delta: 0.1,
        },
        paired_mode: None,
        adapter_policy: None,
        polyx_policy: None,
        n_policy: None,
        contaminant_policy: None,
        raw_backend_report_format: None,
        adapter_preset: None,
        adapter_bank_id: None,
        adapter_bank_hash: None,
        adapter_overrides: None,
    }));
    assert!(record.validate().is_ok());
    let schema = record.metrics.metrics_schema;
    assert_eq!(schema, "fastq_trim_reads_v2");
}

#[test]
fn metrics_schema_rejects_unknown() {
    let mut metrics = metric_set(FastqTrimMetrics {
        reads_in: 100,
        reads_out: 90,
        bases_in: 1000,
        bases_out: 900,
        pairs_in: None,
        pairs_out: None,
        mean_q_before: 30.0,
        mean_q_after: 31.0,
        delta_metrics: FastqDeltaMetrics {
            read_retention: 0.9,
            base_retention: 0.9,
            mean_q_delta: 1.0,
            gc_delta: 0.1,
        },
        paired_mode: None,
        adapter_policy: None,
        polyx_policy: None,
        n_policy: None,
        contaminant_policy: None,
        raw_backend_report_format: None,
        adapter_preset: None,
        adapter_bank_id: None,
        adapter_bank_hash: None,
        adapter_overrides: None,
    });
    metrics.metrics_schema = "fastq_trim_reads_v1".to_string();
    let record = base_record(metrics);
    match record.validate() {
        Ok(()) => panic!("expected schema mismatch"),
        Err(err) => assert!(err.to_string().contains("metric schema mismatch")),
    }
}

#[test]
fn metrics_schema_rejects_mixed_stage() {
    let mut metrics = metric_set(FastqTrimMetrics {
        reads_in: 100,
        reads_out: 90,
        bases_in: 1000,
        bases_out: 900,
        pairs_in: None,
        pairs_out: None,
        mean_q_before: 30.0,
        mean_q_after: 31.0,
        delta_metrics: FastqDeltaMetrics {
            read_retention: 0.9,
            base_retention: 0.9,
            mean_q_delta: 1.0,
            gc_delta: 0.1,
        },
        paired_mode: None,
        adapter_policy: None,
        polyx_policy: None,
        n_policy: None,
        contaminant_policy: None,
        raw_backend_report_format: None,
        adapter_preset: None,
        adapter_bank_id: None,
        adapter_bank_hash: None,
        adapter_overrides: None,
    });
    metrics.metrics_schema = "fastq_validate_reads_v1".to_string();
    let record = base_record(metrics);
    match record.validate() {
        Ok(()) => panic!("expected schema mismatch"),
        Err(err) => assert!(err.to_string().contains("metric schema mismatch")),
    }
}

#[test]
fn execution_metrics_require_positive_values() {
    let mut record = base_record(metric_set(FastqTrimMetrics {
        reads_in: 10,
        reads_out: 9,
        bases_in: 100,
        bases_out: 90,
        pairs_in: None,
        pairs_out: None,
        mean_q_before: 30.0,
        mean_q_after: 31.0,
        delta_metrics: FastqDeltaMetrics {
            read_retention: 0.9,
            base_retention: 0.9,
            mean_q_delta: 1.0,
            gc_delta: 0.1,
        },
        paired_mode: None,
        adapter_policy: None,
        polyx_policy: None,
        n_policy: None,
        contaminant_policy: None,
        raw_backend_report_format: None,
        adapter_preset: None,
        adapter_bank_id: None,
        adapter_bank_hash: None,
        adapter_overrides: None,
    }));
    record.execution.runtime_s = 0.0;
    match record.validate() {
        Ok(()) => panic!("expected runtime error"),
        Err(err) => assert!(err.to_string().contains("runtime_s")),
    }
    record.execution.runtime_s = 1.0;
    record.execution.memory_mb = 0.0;
    match record.validate() {
        Ok(()) => panic!("expected memory error"),
        Err(err) => assert!(err.to_string().contains("memory_mb")),
    }
}

#[test]
#[allow(clippy::too_many_lines)]
fn metrics_schema_matches_stage_and_version_for_all_fastq_stages() {
    let trim = metric_set(FastqTrimMetrics {
        reads_in: 100,
        reads_out: 90,
        bases_in: 1000,
        bases_out: 900,
        pairs_in: None,
        pairs_out: None,
        mean_q_before: 30.0,
        mean_q_after: 31.0,
        delta_metrics: FastqDeltaMetrics {
            read_retention: 0.9,
            base_retention: 0.9,
            mean_q_delta: 1.0,
            gc_delta: 0.1,
        },
        paired_mode: None,
        adapter_policy: None,
        polyx_policy: None,
        n_policy: None,
        contaminant_policy: None,
        raw_backend_report_format: None,
        adapter_preset: None,
        adapter_bank_id: None,
        adapter_bank_hash: None,
        adapter_overrides: None,
    });
    assert_eq!(trim.metrics_schema, "fastq_trim_reads_v2");

    let validate = metric_set(FastqValidateMetrics {
        reads_in: 100,
        reads_out: 100,
        bases_in: 1000,
        bases_out: 1000,
        pairs_in: None,
        pairs_out: None,
        reads_total: 100,
        reads_valid: 90,
        reads_invalid: 10,
        mean_q: 30.0,
        validated_inputs: Some(1),
        validated_pairs: None,
        pair_sync_checked: Some(false),
        pair_sync_pass: None,
        pair_count_match: None,
        strict_pass: Some(true),
        failure_class: Some("none".to_string()),
    });
    assert_eq!(validate.metrics_schema, "fastq_validate_reads_v1");

    let filter = metric_set(FastqFilterMetrics {
        reads_in: 100,
        reads_out: 90,
        reads_dropped: 10,
        reads_removed_by_n: 0,
        reads_removed_by_entropy: 0,
        reads_removed_low_complexity: 0,
        reads_removed_by_kmer: 0,
        reads_removed_contaminant_kmer: 0,
        reads_removed_by_length: 0,
        bases_in: 1000,
        bases_out: 900,
        pairs_in: None,
        pairs_out: None,
        mean_q_before: 30.0,
        mean_q_after: 31.0,
        delta_metrics: FastqDeltaMetrics {
            read_retention: 0.9,
            base_retention: 0.9,
            mean_q_delta: 1.0,
            gc_delta: 0.1,
        },
    });
    assert_eq!(filter.metrics_schema, "fastq_filter_reads_v2");

    let low_complexity = metric_set(FastqLowComplexityMetrics {
        reads_in: 100,
        reads_out: 92,
        bases_in: 1000,
        bases_out: 910,
        reads_removed_low_complexity: 8,
        mean_q_before: 30.0,
        mean_q_after: 30.5,
        delta_metrics: FastqDeltaMetrics {
            read_retention: 0.92,
            base_retention: 0.91,
            mean_q_delta: 0.5,
            gc_delta: 0.01,
        },
    });
    assert_eq!(
        low_complexity.metrics_schema,
        "fastq_filter_low_complexity_v1"
    );

    let deduplicate = metric_set(FastqDuplicateMetrics {
        reads_in: 100,
        reads_out: 92,
        duplicates_removed: 8,
        dedup_rate: 0.08,
        tool: Some("fastuniq".to_string()),
        paired_mode: Some("paired_end".to_string()),
        dedup_mode: Some("exact".to_string()),
        keep_order: Some(true),
        pair_count_match: Some(true),
        duplicate_class_count: Some(1),
        duplicate_provenance_json: Some("out/duplicate_provenance.json".to_string()),
        raw_backend_report_format: Some("fastuniq_log".to_string()),
    });
    assert_eq!(deduplicate.metrics_schema, "fastq_remove_duplicates_v1");

    let merge = metric_set(FastqMergeMetrics {
        reads_in: 100,
        reads_out: 80,
        bases_in: 1000,
        bases_out: 800,
        pairs_in: 100,
        pairs_out: 80,
        reads_r1: 100,
        reads_r2: 100,
        reads_merged: 80,
        reads_unmerged: 10,
        merge_rate: 0.8,
    });
    assert_eq!(merge.metrics_schema, "fastq_merge_pairs_v1");

    let correct = metric_set(FastqCorrectMetrics {
        reads_in: 100,
        reads_out: 100,
        bases_in: 1000,
        bases_out: 900,
        pairs_in: None,
        pairs_out: None,
        mean_q_before: 30.0,
        mean_q_after: 31.0,
        kmer_fix_rate: 0.5,
    });
    assert_eq!(correct.metrics_schema, "fastq_correct_errors_v1");

    let qc_post = metric_set(FastqQcPostMetrics {
        reads_in: 100,
        reads_out: 100,
        bases_in: 1000,
        bases_out: 1000,
        pairs_in: None,
        pairs_out: None,
        mean_q: 30.0,
        contamination_rate: 0.1,
        aggregation_engine: Some("multiqc".to_string()),
        aggregation_scope: Some("governed_qc_artifacts".to_string()),
        governed_qc_input_count: Some(2),
        governed_qc_contributor_stage_ids: bijux_dna_analyze::model::JsonBlob::from(
            serde_json::json!(["fastq.trim_reads", "fastq.validate_reads"]),
        ),
        governed_qc_contributor_tool_ids: bijux_dna_analyze::model::JsonBlob::from(
            serde_json::json!(["fastp", "fastqvalidator"]),
        ),
        governed_qc_lineage_hash: Some("lineage".to_string()),
        multiqc_sample_count: Some(2),
        multiqc_module_count: Some(5),
        raw_fastqc_dir: None,
        trimmed_fastqc_dir: None,
        multiqc_report: None,
        multiqc_data: None,
    });
    assert_eq!(qc_post.metrics_schema, "fastq_report_qc_v1");

    let umi = metric_set(FastqUmiMetrics {
        reads_in: 100,
        reads_out: 80,
        bases_in: 1000,
        bases_out: 800,
        pairs_in: None,
        pairs_out: None,
        reads_with_umi: 75,
    });
    assert_eq!(umi.metrics_schema, "fastq_extract_umis_v1");

    let screen = metric_set(FastqScreenMetrics {
        reads_in: 100,
        reads_out: 100,
        bases_in: 1000,
        bases_out: 1000,
        pairs_in: 0,
        pairs_out: 0,
        contamination_rate: 0.1,
        classified_fraction: Some(0.9),
        unclassified_fraction: Some(0.1),
        classifier: Some("kraken2".to_string()),
        report_format: Some("kraken_report".to_string()),
        database_catalog_id: Some("taxonomy_reference".to_string()),
        database_artifact_id: Some("taxonomy_db".to_string()),
        minimum_confidence: Some(0.05),
        emit_unclassified: Some(true),
        contamination_summary: bijux_dna_analyze::model::JsonBlob::default(),
        top_taxa: bijux_dna_analyze::model::JsonBlob::default(),
    });
    assert_eq!(screen.metrics_schema, "fastq_screen_taxonomy_v1");

    let stats = metric_set(FastqStatsMetrics {
        reads_total: 100,
        bases_total: 1000,
        mean_q: 30.0,
        gc_percent: 50.0,
        length_histogram: vec![LengthHistogramBin {
            length: 100,
            count: 100,
        }],
    });
    assert_eq!(stats.metrics_schema, "fastq_profile_reads_v1");

    let index_reference = metric_set(FastqIndexReferenceMetrics {
        reference_bytes: 4096,
        index_bytes: 3072,
        index_file_count: 2,
    });
    assert_eq!(index_reference.metrics_schema, "fastq_index_reference_v1");

    let deplete_host = metric_set(FastqDepleteHostMetrics {
        reads_in: 100,
        reads_out: 75,
        bases_in: 1000,
        bases_out: 760,
        pairs_in: 50,
        pairs_out: 38,
        host_fraction_removed: 0.25,
        depletion_summary: bijux_dna_analyze::model::JsonBlob::default(),
    });
    assert_eq!(deplete_host.metrics_schema, "fastq_deplete_host_v1");

    let deplete_reference_contaminants = metric_set(FastqDepleteReferenceContaminantsMetrics {
        reads_in: 100,
        reads_out: 72,
        bases_in: 1000,
        bases_out: 710,
        pairs_in: 50,
        pairs_out: 36,
        contaminant_fraction_removed: 0.28,
        depletion_summary: bijux_dna_analyze::model::JsonBlob::default(),
    });
    assert_eq!(
        deplete_reference_contaminants.metrics_schema,
        "fastq_deplete_reference_contaminants_v1"
    );
}
