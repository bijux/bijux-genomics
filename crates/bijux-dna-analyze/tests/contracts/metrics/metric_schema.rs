use bijux_dna_analyze::{
    metric_set, BenchmarkRecord, FastqCorrectMetrics, FastqDeltaMetrics, FastqFilterMetrics,
    FastqMergeMetrics, FastqQcPostMetrics, FastqScreenMetrics, FastqStatsMetrics, FastqTrimMetrics,
    FastqUmiMetrics, FastqValidateMetrics, LengthHistogramBin, MetricSet,
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
        dedup_rate: 0.2,
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
}
