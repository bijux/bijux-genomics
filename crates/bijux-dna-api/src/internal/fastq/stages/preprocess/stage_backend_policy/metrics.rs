use super::StageResultV1;

pub(crate) fn canonical_sample_identity(sample_id: &str) -> String {
    let mut out = String::with_capacity(sample_id.len());
    for ch in sample_id.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' {
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push('_');
        }
    }
    out.trim_matches('_').to_string()
}

pub(crate) fn parse_first_u64_after_key(text: &str, key: &str) -> Option<u64> {
    for line in text.lines() {
        if !line.to_ascii_lowercase().contains(&key.to_ascii_lowercase()) {
            continue;
        }
        let digits: String = line.chars().filter(char::is_ascii_digit).collect();
        if let Ok(parsed) = digits.parse::<u64>() {
            return Some(parsed);
        }
    }
    None
}

pub(crate) fn parse_validate_reads_metrics(
    out_dir: &std::path::Path,
    execution: &StageResultV1,
) -> serde_json::Value {
    let report_path = out_dir.join("validation.json");
    if let Ok(raw) = std::fs::read_to_string(&report_path) {
        if let Ok(report) = bijux_dna_domain_fastq::observer::parse_validation_report(&raw) {
            return serde_json::json!({
                "schema_version": "bijux.fastq_stage_metrics.v1",
                "stage": "fastq.validate_reads",
                "validator": report.tool_id,
                "validation_mode": report.validation_mode,
                "pair_sync_policy": report.pair_sync_policy,
                "validated_inputs": report.validated_inputs,
                "validated_reads_r1": report.validated_reads_r1,
                "validated_reads_r2": report.validated_reads_r2,
                "validated_pairs": report.validated_pairs,
                "status_r1": report.status_r1,
                "status_r2": report.status_r2,
                "pair_sync_checked": report.pair_sync_checked,
                "pair_sync_pass": report.pair_sync_pass,
                "pair_count_match": report.pair_count_match,
                "failure_class": report.failure_class,
                "strict_pass": report.strict_pass,
                "exit_code": report.exit_code,
                "report_json": report_path,
            });
        }
    }

    let merged = format!("{}\n{}", execution.stdout, execution.stderr);
    serde_json::json!({
        "schema_version": "bijux.fastq_stage_metrics.v1",
        "stage": "fastq.validate_reads",
        "validator": "tool_stdout_stderr_parser",
        "validated_inputs": parse_first_u64_after_key(&merged, "read")
            .or_else(|| parse_first_u64_after_key(&merged, "sequences")),
        "failure_class": serde_json::Value::Null,
        "strict_pass": execution.exit_code == 0,
        "exit_code": execution.exit_code,
    })
}

pub(crate) fn parse_profile_reads_metrics(out_dir: &std::path::Path) -> serde_json::Value {
    let report_path = out_dir.join("qc.json");
    if let Ok(raw) = std::fs::read_to_string(&report_path) {
        if let Ok(report) = bijux_dna_domain_fastq::observer::parse_profile_reads_report(&raw) {
            return serde_json::json!({
                "schema_version": "bijux.fastq_stage_metrics.v1",
                "stage": "fastq.profile_reads",
                "tool": report.tool_id,
                "paired_mode": report.paired_mode,
                "threads": report.threads,
                "reads_total": report.reads_total,
                "bases_total": report.bases_total,
                "mean_q": report.mean_q,
                "gc_percent": report.gc_percent,
                "length_histogram_bins": report.length_histogram.len(),
                "length_histogram_source": report.length_histogram_source,
                "mate_summary_count": report.mate_summaries.len(),
                "qc_tsv": report.qc_tsv,
                "qc_plots_dir": report.qc_plots_dir,
                "raw_backend_report": report.raw_backend_report,
                "raw_backend_report_format": report.raw_backend_report_format,
                "report_json": report_path,
            });
        }
    }
    serde_json::json!({
        "schema_version": "bijux.fastq_stage_metrics.v1",
        "stage": "fastq.profile_reads",
        "tool": "report_missing",
        "reads_total": serde_json::Value::Null,
        "bases_total": serde_json::Value::Null,
        "report_json": report_path,
    })
}

pub(crate) fn parse_detect_duplicates_premerge_metrics(
    out_dir: &std::path::Path,
) -> serde_json::Value {
    let report_path = out_dir.join("duplicate_signal_report.json");
    if let Ok(raw) = std::fs::read_to_string(&report_path) {
        if let Ok(report) =
            bijux_dna_domain_fastq::observer::parse_detect_duplicates_premerge_report(&raw)
        {
            return serde_json::json!({
                "schema_version": "bijux.fastq_stage_metrics.v1",
                "stage": "fastq.detect_duplicates_premerge",
                "tool": report.tool_id,
                "paired_mode": report.paired_mode,
                "duplicate_detection_policy": report.duplicate_detection_policy,
                "measurement_scope": report.measurement_scope,
                "modifies_reads": report.modifies_reads,
                "advisory_only": report.advisory_only,
                "reads_in": report.reads_in,
                "duplicate_count": report.duplicate_signal_reads,
                "duplicate_fraction": report.duplicate_signal_fraction,
                "inspected_pair_count": report.compared_read_pairs,
                "report_json": report_path,
            });
        }
    }
    serde_json::json!({
        "schema_version": "bijux.fastq_stage_metrics.v1",
        "stage": "fastq.detect_duplicates_premerge",
        "tool": "report_missing",
        "duplicate_count": serde_json::Value::Null,
        "duplicate_fraction": serde_json::Value::Null,
        "inspected_pair_count": serde_json::Value::Null,
        "report_json": report_path,
    })
}

pub(crate) fn parse_estimate_library_complexity_prealign_metrics(
    out_dir: &std::path::Path,
) -> serde_json::Value {
    let report_path = out_dir.join("library_complexity_report.json");
    if let Ok(raw) = std::fs::read_to_string(&report_path) {
        if let Ok(report) =
            bijux_dna_domain_fastq::observer::parse_estimate_library_complexity_prealign_report(
                &raw,
            )
        {
            let estimated_complexity = if report.insufficient_data_reason.is_some() {
                serde_json::Value::Null
            } else {
                serde_json::json!(report.estimated_unique_fraction)
            };
            let complexity_status = if report.insufficient_data_reason.is_some() {
                "insufficient_data"
            } else {
                "complexity_estimated"
            };
            return serde_json::json!({
                "schema_version": "bijux.fastq_stage_metrics.v1",
                "stage": "fastq.estimate_library_complexity_prealign",
                "tool": report.tool_id,
                "paired_mode": report.paired_mode,
                "complexity_policy": report.complexity_policy,
                "estimate_method": report.estimate_method,
                "modifies_reads": report.modifies_reads,
                "advisory_only": report.advisory_only,
                "reads_in": report.reads_in,
                "estimated_complexity": estimated_complexity,
                "estimated_unique_fraction": report.estimated_unique_fraction,
                "estimated_duplicate_fraction": report.estimated_duplicate_fraction,
                "insufficient_data_reason": report.insufficient_data_reason,
                "complexity_status": complexity_status,
                "kmer_size": report.kmer_size,
                "report_json": report_path,
            });
        }
    }
    serde_json::json!({
        "schema_version": "bijux.fastq_stage_metrics.v1",
        "stage": "fastq.estimate_library_complexity_prealign",
        "tool": "report_missing",
        "estimated_complexity": serde_json::Value::Null,
        "estimated_duplicate_fraction": serde_json::Value::Null,
        "insufficient_data_reason": serde_json::Value::Null,
        "complexity_status": "report_missing",
        "report_json": report_path,
    })
}

pub(crate) fn parse_filter_reads_metrics(out_dir: &std::path::Path) -> serde_json::Value {
    let report_path = out_dir.join("filter_report.json");
    if let Ok(raw) = std::fs::read_to_string(&report_path) {
        if let Ok(report) = bijux_dna_domain_fastq::observer::parse_filter_reads_report(&raw) {
            return serde_json::json!({
                "schema_version": "bijux.fastq_stage_metrics.v1",
                "stage": "fastq.filter_reads",
                "tool": report.tool_id,
                "paired_mode": report.paired_mode,
                "threads": report.threads,
                "filtered_reads_r1": report.output_r1,
                "filtered_reads_r2": report.output_r2,
                "max_n": report.max_n,
                "max_n_fraction": report.max_n_fraction,
                "max_n_count": report.max_n_count,
                "low_complexity_threshold": report.low_complexity_threshold,
                "entropy_threshold": report.entropy_threshold,
                "n_policy": report.n_policy,
                "polyx_policy": report.polyx_policy,
                "contaminant_db": report.contaminant_db,
                "reads_in": report.reads_in,
                "reads_out": report.reads_out,
                "reads_dropped": report.reads_dropped,
                "reads_retained": report.reads_out,
                "reads_removed": report.reads_dropped,
                "reads_removed_by_n": report.reads_removed_by_n,
                "reads_removed_by_entropy": report.reads_removed_by_entropy,
                "reads_removed_low_complexity": report.reads_removed_low_complexity,
                "reads_removed_by_kmer": report.reads_removed_by_kmer,
                "reads_removed_contaminant_kmer": report.reads_removed_contaminant_kmer,
                "reads_removed_by_length": report.reads_removed_by_length,
                "bases_in": report.bases_in,
                "bases_out": report.bases_out,
                "pairs_in": report.pairs_in,
                "pairs_out": report.pairs_out,
                "mean_q_before": report.mean_q_before,
                "mean_q_after": report.mean_q_after,
                "runtime_s": report.runtime_s,
                "memory_mb": report.memory_mb,
                "raw_backend_report": report.raw_backend_report,
                "raw_backend_report_format": report.raw_backend_report_format,
                "report_json": report_path,
            });
        }
    }
    serde_json::json!({
        "schema_version": "bijux.fastq_stage_metrics.v1",
        "stage": "fastq.filter_reads",
        "tool": "report_missing",
        "filtered_reads_r1": serde_json::Value::Null,
        "filtered_reads_r2": serde_json::Value::Null,
        "reads_in": serde_json::Value::Null,
        "reads_out": serde_json::Value::Null,
        "reads_dropped": serde_json::Value::Null,
        "reads_retained": serde_json::Value::Null,
        "reads_removed": serde_json::Value::Null,
        "report_json": report_path,
    })
}

pub(crate) fn parse_correct_errors_metrics(out_dir: &std::path::Path) -> serde_json::Value {
    let report_path = out_dir.join("correct_report.json");
    if let Ok(raw) = std::fs::read_to_string(&report_path) {
        if let Ok(report) = bijux_dna_domain_fastq::observer::parse_correct_errors_report(&raw) {
            return serde_json::json!({
                "schema_version": "bijux.fastq_stage_metrics.v1",
                "stage": "fastq.correct_errors",
                "tool": report.tool_id,
                "paired_mode": report.paired_mode,
                "threads": report.threads,
                "correction_engine": report.correction_engine,
                "quality_encoding": report.quality_encoding,
                "kmer_size": report.kmer_size,
                "genome_size": report.genome_size,
                "max_memory_gb": report.max_memory_gb,
                "trusted_kmer_artifact": report.trusted_kmer_artifact,
                "conservative_mode": report.conservative_mode,
                "corrected_reads_r1": report.output_r1,
                "corrected_reads_r2": report.output_r2,
                "corrected_reads": report.corrected_reads,
                "changed_reads": report.changed_reads,
                "unchanged_reads": report.unchanged_reads,
                "reads_in": report.reads_in,
                "reads_out": report.reads_out,
                "bases_in": report.bases_in,
                "bases_out": report.bases_out,
                "pairs_in": report.pairs_in,
                "pairs_out": report.pairs_out,
                "mean_q_before": report.mean_q_before,
                "mean_q_after": report.mean_q_after,
                "kmer_fix_rate": report.kmer_fix_rate,
                "correction_effect": report.correction_effect,
                "raw_backend_report": report.raw_backend_report,
                "raw_backend_report_format": report.raw_backend_report_format,
                "report_json": report_path,
            });
        }
    }
    serde_json::json!({
        "schema_version": "bijux.fastq_stage_metrics.v1",
        "stage": "fastq.correct_errors",
        "tool": "report_missing",
        "corrected_reads_r1": serde_json::Value::Null,
        "corrected_reads_r2": serde_json::Value::Null,
        "correction_engine": serde_json::Value::Null,
        "corrected_reads": serde_json::Value::Null,
        "changed_reads": serde_json::Value::Null,
        "unchanged_reads": serde_json::Value::Null,
        "kmer_fix_rate": serde_json::Value::Null,
        "report_json": report_path,
    })
}

pub(crate) fn parse_filter_low_complexity_metrics(out_dir: &std::path::Path) -> serde_json::Value {
    let report_path = out_dir.join("low_complexity_report.json");
    if let Ok(raw) = std::fs::read_to_string(&report_path) {
        if let Ok(report) =
            bijux_dna_domain_fastq::observer::parse_filter_low_complexity_report(&raw)
        {
            return serde_json::json!({
                "schema_version": "bijux.fastq_stage_metrics.v1",
                "stage": "fastq.filter_low_complexity",
                "tool": report.tool_id,
                "paired_mode": report.paired_mode,
                "threads": report.threads,
                "filtered_fastq_r1": report.output_r1,
                "filtered_fastq_r2": report.output_r2,
                "entropy_threshold": report.entropy_threshold,
                "polyx_threshold": report.polyx_threshold,
                "reads_in": report.reads_in,
                "reads_out": report.reads_out,
                "reads_dropped": report.reads_in.saturating_sub(report.reads_out),
                "reads_retained": report.reads_out,
                "reads_removed": report.reads_in.saturating_sub(report.reads_out),
                "reads_removed_low_complexity": report.reads_removed_low_complexity,
                "bases_in": report.bases_in,
                "bases_out": report.bases_out,
                "pairs_in": report.pairs_in,
                "pairs_out": report.pairs_out,
                "mean_q_before": report.mean_q_before,
                "mean_q_after": report.mean_q_after,
                "raw_backend_report": report.raw_backend_report,
                "raw_backend_report_format": report.raw_backend_report_format,
                "report_json": report_path,
            });
        }
    }
    serde_json::json!({
        "schema_version": "bijux.fastq_stage_metrics.v1",
        "stage": "fastq.filter_low_complexity",
        "tool": "report_missing",
        "filtered_fastq_r1": serde_json::Value::Null,
        "filtered_fastq_r2": serde_json::Value::Null,
        "reads_in": serde_json::Value::Null,
        "reads_out": serde_json::Value::Null,
        "reads_dropped": serde_json::Value::Null,
        "reads_retained": serde_json::Value::Null,
        "reads_removed": serde_json::Value::Null,
        "reads_removed_low_complexity": serde_json::Value::Null,
        "report_json": report_path,
    })
}

pub(crate) fn parse_profile_read_lengths_metrics(out_dir: &std::path::Path) -> serde_json::Value {
    let report_path = out_dir.join("profile_read_lengths_report.json");
    if let Ok(raw) = std::fs::read_to_string(&report_path) {
        if let Ok(report) =
            bijux_dna_domain_fastq::observer::parse_profile_read_lengths_report(&raw)
        {
            return serde_json::json!({
                "schema_version": "bijux.fastq_stage_metrics.v1",
                "stage": "fastq.profile_read_lengths",
                "tool": report.tool_id,
                "paired_mode": report.paired_mode,
                "threads": report.threads,
                "histogram_bins": report.histogram_bins,
                "read_count": report.read_count,
                "min_read_length": report.min_read_length,
                "mean_read_length": report.mean_read_length,
                "median_read_length": report.median_read_length,
                "max_read_length": report.max_read_length,
                "distinct_lengths": report.distinct_lengths,
                "histogram_entry_count": report.histogram.len(),
                "length_distribution_tsv": report.length_distribution_tsv,
                "length_distribution_json": report.length_distribution_json,
                "raw_backend_report": report.raw_backend_report,
                "raw_backend_report_format": report.raw_backend_report_format,
                "report_json": report_path,
            });
        }
    }
    serde_json::json!({
        "schema_version": "bijux.fastq_stage_metrics.v1",
        "stage": "fastq.profile_read_lengths",
        "tool": "report_missing",
        "read_count": serde_json::Value::Null,
        "min_read_length": serde_json::Value::Null,
        "mean_read_length": serde_json::Value::Null,
        "median_read_length": serde_json::Value::Null,
        "report_json": report_path,
    })
}

pub(crate) fn parse_profile_overrepresented_metrics(
    out_dir: &std::path::Path,
) -> serde_json::Value {
    let report_path = out_dir.join("overrepresented_report.json");
    if let Ok(raw) = std::fs::read_to_string(&report_path) {
        if let Ok(report) =
            bijux_dna_domain_fastq::observer::parse_profile_overrepresented_report(&raw)
        {
            return serde_json::json!({
                "schema_version": "bijux.fastq_stage_metrics.v1",
                "stage": "fastq.profile_overrepresented_sequences",
                "tool": report.tool_id,
                "paired_mode": report.paired_mode,
                "threads": report.threads,
                "top_k": report.top_k,
                "sequence_count": report.sequence_count,
                "flagged_sequences": report.flagged_sequences,
                "top_fraction": report.top_fraction,
                "row_count": report.rows.len(),
                "overrepresented_sequences_tsv": report.overrepresented_sequences_tsv,
                "overrepresented_sequences_json": report.overrepresented_sequences_json,
                "raw_backend_report": report.raw_backend_report,
                "raw_backend_report_format": report.raw_backend_report_format,
                "report_json": report_path,
            });
        }
    }
    serde_json::json!({
        "schema_version": "bijux.fastq_stage_metrics.v1",
        "stage": "fastq.profile_overrepresented_sequences",
        "tool": "report_missing",
        "sequence_count": serde_json::Value::Null,
        "flagged_sequences": serde_json::Value::Null,
        "top_fraction": serde_json::Value::Null,
        "report_json": report_path,
    })
}

pub(crate) fn parse_infer_asvs_metrics(out_dir: &std::path::Path) -> serde_json::Value {
    let report_path = out_dir.join("infer_asvs_report.json");
    if let Ok(raw) = std::fs::read_to_string(&report_path) {
        if let Ok(report) = bijux_dna_domain_fastq::observer::parse_infer_asvs_report(&raw) {
            return serde_json::json!({
                "schema_version": "bijux.fastq_stage_metrics.v1",
                "stage": "fastq.infer_asvs",
                "tool": report.tool_id,
                "paired_mode": report.paired_mode,
                "denoising_method": report.denoising_method,
                "pooling_mode": report.pooling_mode,
                "chimera_policy": report.chimera_policy,
                "asv_table_tsv": report.asv_table_tsv,
                "representative_sequences_fasta": report.asv_sequences_fasta,
                "asv_count": report.asv_count,
                "sample_count": report.sample_count,
                "representative_sequence_count": report.representative_sequence_count,
                "used_fallback": report.used_fallback,
                "raw_backend_report_format": report.raw_backend_report_format,
                "report_json": report_path,
            });
        }
    }
    serde_json::json!({
        "schema_version": "bijux.fastq_stage_metrics.v1",
        "stage": "fastq.infer_asvs",
        "tool": "report_missing",
        "asv_table_tsv": serde_json::Value::Null,
        "representative_sequences_fasta": serde_json::Value::Null,
        "asv_count": serde_json::Value::Null,
        "sample_count": serde_json::Value::Null,
        "representative_sequence_count": serde_json::Value::Null,
        "report_json": report_path,
    })
}

pub(crate) fn parse_extract_umis_metrics(out_dir: &std::path::Path) -> serde_json::Value {
    let report_path = out_dir.join("umi_report.json");
    if let Ok(raw) = std::fs::read_to_string(&report_path) {
        if let Ok(report) = bijux_dna_domain_fastq::observer::parse_extract_umis_report(&raw) {
            let umi_summary = report.canonical_umi_summary();
            return serde_json::json!({
                "schema_version": "bijux.fastq_stage_metrics.v1",
                "stage": "fastq.extract_umis",
                "tool": report.tool_id,
                "paired_mode": report.paired_mode,
                "threads": report.threads,
                "umi_pattern": report.umi_pattern,
                "extraction_location": report.extraction_location,
                "read_name_transform": report.read_name_transform,
                "tag_header_format": umi_summary.tag_header_format,
                "failed_extraction_policy": report.failed_extraction_policy,
                "downstream_propagation": report.downstream_propagation,
                "reads_in": report.reads_in,
                "reads_out": report.reads_out,
                "bases_in": report.bases_in,
                "bases_out": report.bases_out,
                "pairs_in": report.pairs_in,
                "pairs_out": report.pairs_out,
                "reads_with_umi": report.reads_with_umi,
                "failed_extractions": report.failed_extractions,
                "extracted_umi_count": umi_summary.extracted_umi_count,
                "invalid_umi_count": umi_summary.invalid_umi_count,
                "mean_q_before": report.mean_q_before,
                "mean_q_after": report.mean_q_after,
                "raw_backend_report": report.raw_backend_report,
                "raw_backend_report_format": report.raw_backend_report_format,
                "report_json": report_path,
            });
        }
    }
    serde_json::json!({
        "schema_version": "bijux.fastq_stage_metrics.v1",
        "stage": "fastq.extract_umis",
        "tool": "report_missing",
        "umi_pattern": serde_json::Value::Null,
        "extraction_location": serde_json::Value::Null,
        "read_name_transform": serde_json::Value::Null,
        "tag_header_format": serde_json::Value::Null,
        "failed_extraction_policy": serde_json::Value::Null,
        "downstream_propagation": serde_json::Value::Null,
        "reads_in": serde_json::Value::Null,
        "reads_out": serde_json::Value::Null,
        "reads_with_umi": serde_json::Value::Null,
        "failed_extractions": serde_json::Value::Null,
        "extracted_umi_count": serde_json::Value::Null,
        "invalid_umi_count": serde_json::Value::Null,
        "report_json": report_path,
    })
}

pub(crate) fn parse_trim_terminal_damage_metrics(out_dir: &std::path::Path) -> serde_json::Value {
    let report_path = out_dir.join("trim_terminal_damage_report.json");
    if let Ok(raw) = std::fs::read_to_string(&report_path) {
        if let Ok(report) = bijux_dna_domain_fastq::observer::parse_terminal_damage_report(&raw) {
            return serde_json::json!({
                "schema_version": "bijux.fastq_stage_metrics.v1",
                "stage": "fastq.trim_terminal_damage",
                "tool": report.tool_id,
                "paired_mode": report.paired_mode,
                "threads": report.threads,
                "reads_in": report.reads_in,
                "reads_out": report.reads_out,
                "reads_retained": report.reads_out,
                "bases_in": report.bases_in,
                "bases_out": report.bases_out,
                "bases_removed": report.bases_in.zip(report.bases_out).map(|(bases_in, bases_out)| {
                    bases_in.saturating_sub(bases_out)
                }),
                "damage_mode": report.damage_mode,
                "execution_policy": report.execution_policy,
                "trim_5p_bases": report.trim_5p_bases,
                "trim_3p_bases": report.trim_3p_bases,
                "requested_trim_5p_bases": report.requested_trim_5p_bases,
                "requested_trim_3p_bases": report.requested_trim_3p_bases,
                "udg_classification": report.udg_classification,
                "ct_ga_asymmetry_pre": report.ct_ga_asymmetry_pre,
                "ct_ga_asymmetry_post": report.ct_ga_asymmetry_post,
                "used_fallback": report.used_fallback,
                "backend_metrics": report.backend_metrics,
                "raw_backend_report": report.raw_backend_report,
                "raw_backend_report_format": report.raw_backend_report_format,
                "report_json": report_path,
            });
        }
    }
    serde_json::json!({
        "schema_version": "bijux.fastq_stage_metrics.v1",
        "stage": "fastq.trim_terminal_damage",
        "tool": "report_missing",
        "reads_retained": serde_json::Value::Null,
        "bases_removed": serde_json::Value::Null,
        "udg_classification": serde_json::Value::Null,
        "ct_ga_asymmetry_pre": serde_json::Value::Null,
        "ct_ga_asymmetry_post": serde_json::Value::Null,
        "report_json": report_path,
    })
}

pub(crate) fn parse_trim_reads_metrics(out_dir: &std::path::Path) -> serde_json::Value {
    let report_path = out_dir.join("trim_report.json");
    if let Ok(raw) = std::fs::read_to_string(&report_path) {
        if let Ok(report) = bijux_dna_domain_fastq::observer::parse_trim_reads_report(&raw) {
            let reads_retained = report.reads_out;
            let reads_dropped = report
                .reads_in
                .zip(report.reads_out)
                .map(|(reads_in, reads_out)| reads_in.saturating_sub(reads_out));
            let bases_removed = report
                .bases_in
                .zip(report.bases_out)
                .map(|(bases_in, bases_out)| bases_in.saturating_sub(bases_out));
            return serde_json::json!({
                "schema_version": "bijux.fastq_stage_metrics.v1",
                "stage": "fastq.trim_reads",
                "tool": report.tool_id,
                "paired_mode": report.paired_mode,
                "threads": report.threads,
                "min_length": report.min_length,
                "quality_cutoff": report.quality_cutoff,
                "adapter_policy": report.adapter_policy,
                "adapter_overrides": report.adapter_overrides,
                "polyx_policy": report.polyx_policy,
                "n_policy": report.n_policy,
                "contaminant_policy": report.contaminant_policy,
                "adapter_bank_id": report.adapter_bank_id,
                "adapter_bank_hash": report.adapter_bank_hash,
                "adapter_preset": report.adapter_preset,
                "polyx_bank_id": report.polyx_bank_id,
                "polyx_bank_hash": report.polyx_bank_hash,
                "polyx_preset": report.polyx_preset,
                "contaminant_bank_id": report.contaminant_bank_id,
                "contaminant_bank_hash": report.contaminant_bank_hash,
                "contaminant_preset": report.contaminant_preset,
                "trimmed_reads_r1": report.output_r1,
                "trimmed_reads_r2": report.output_r2,
                "report_json": report_path,
                "reads_in": report.reads_in,
                "reads_out": report.reads_out,
                "reads_retained": reads_retained,
                "reads_dropped": reads_dropped,
                "bases_in": report.bases_in,
                "bases_out": report.bases_out,
                "bases_removed": bases_removed,
                "pairs_in": report.pairs_in,
                "pairs_out": report.pairs_out,
                "mean_q_before": report.mean_q_before,
                "mean_q_after": report.mean_q_after,
                "runtime_s": report.runtime_s,
                "memory_mb": report.memory_mb,
                "raw_backend_report": report.raw_backend_report,
                "raw_backend_report_format": report.raw_backend_report_format,
            });
        }
    }
    serde_json::json!({
        "schema_version": "bijux.fastq_stage_metrics.v1",
        "stage": "fastq.trim_reads",
        "tool": "report_missing",
        "report_json": report_path,
    })
}

pub(crate) fn parse_merge_pairs_metrics(out_dir: &std::path::Path) -> serde_json::Value {
    let report_path = out_dir.join("merge_report.json");
    if let Ok(raw) = std::fs::read_to_string(&report_path) {
        if let Ok(report) = bijux_dna_domain_fastq::observer::parse_merge_pairs_report(&raw) {
            let pair_counts = report.canonical_pair_counts();
            return serde_json::json!({
                "schema_version": "bijux.fastq_stage_metrics.v1",
                "stage": "fastq.merge_pairs",
                "tool": report.tool_id,
                "paired_mode": report.paired_mode,
                "merge_engine": report.merge_engine,
                "threads": report.threads,
                "merge_overlap": report.merge_overlap,
                "min_length": report.min_len,
                "unmerged_read_policy": report.unmerged_read_policy,
                "reads_r1": report.reads_r1,
                "reads_r2": report.reads_r2,
                "input_pair_count": pair_counts.input_pair_count,
                "reads_merged": report.reads_merged,
                "reads_unmerged": report.reads_unmerged,
                "merged_pair_count": pair_counts.merged_pair_count,
                "unmerged_pair_count": pair_counts.unmerged_pair_count,
                "discarded_pair_count": pair_counts.discarded_pair_count,
                "merge_rate": report.merge_rate,
                "runtime_s": report.runtime_s,
                "memory_mb": report.memory_mb,
                "raw_backend_report": report.raw_backend_report,
                "raw_backend_report_format": report.raw_backend_report_format,
                "report_json": report_path,
            });
        }
    }
    serde_json::json!({
        "schema_version": "bijux.fastq_stage_metrics.v1",
        "stage": "fastq.merge_pairs",
        "tool": "report_missing",
        "paired_mode": serde_json::Value::Null,
        "merge_engine": serde_json::Value::Null,
        "threads": serde_json::Value::Null,
        "merge_overlap": serde_json::Value::Null,
        "min_length": serde_json::Value::Null,
        "unmerged_read_policy": serde_json::Value::Null,
        "reads_r1": serde_json::Value::Null,
        "reads_r2": serde_json::Value::Null,
        "input_pair_count": serde_json::Value::Null,
        "reads_merged": serde_json::Value::Null,
        "reads_unmerged": serde_json::Value::Null,
        "merged_pair_count": serde_json::Value::Null,
        "unmerged_pair_count": serde_json::Value::Null,
        "discarded_pair_count": serde_json::Value::Null,
        "merge_rate": serde_json::Value::Null,
        "runtime_s": serde_json::Value::Null,
        "memory_mb": serde_json::Value::Null,
        "raw_backend_report": serde_json::Value::Null,
        "raw_backend_report_format": serde_json::Value::Null,
        "report_json": report_path,
    })
}

pub(crate) fn parse_cluster_otus_metrics(out_dir: &std::path::Path) -> serde_json::Value {
    let report_path = out_dir.join("cluster_otus_report.json");
    if let Ok(raw) = std::fs::read_to_string(&report_path) {
        if let Ok(report) = bijux_dna_domain_fastq::observer::parse_cluster_otus_report(&raw) {
            return serde_json::json!({
                "schema_version": "bijux.fastq_stage_metrics.v1",
                "stage": "fastq.cluster_otus",
                "tool": report.tool_id,
                "otu_identity": report.otu_identity,
                "clustering_threshold": report.otu_identity,
                "threads": report.threads,
                "otu_table_tsv": report.otu_table,
                "representative_sequences_fasta": report.otu_representatives,
                "otu_count": report.otu_count,
                "sample_count": report.sample_count,
                "representative_sequence_count": report.representative_sequence_count,
                "output_table_kind": report.output_table_kind,
                "runtime_s": report.runtime_s,
                "memory_mb": report.memory_mb,
                "used_fallback": report.used_fallback,
                "raw_backend_report": report.raw_backend_report,
                "raw_backend_report_format": report.raw_backend_report_format,
                "report_json": report_path,
            });
        }
    }
    serde_json::json!({
        "schema_version": "bijux.fastq_stage_metrics.v1",
        "stage": "fastq.cluster_otus",
        "tool": "report_missing",
        "clustering_threshold": serde_json::Value::Null,
        "otu_table_tsv": serde_json::Value::Null,
        "representative_sequences_fasta": serde_json::Value::Null,
        "otu_count": serde_json::Value::Null,
        "sample_count": serde_json::Value::Null,
        "representative_sequence_count": serde_json::Value::Null,
        "report_json": report_path,
    })
}

pub(crate) fn parse_remove_duplicates_metrics(out_dir: &std::path::Path) -> serde_json::Value {
    let report_path = out_dir.join("deduplicate_report.json");
    if let Ok(raw) = std::fs::read_to_string(&report_path) {
        if let Ok(report) = bijux_dna_domain_fastq::observer::parse_remove_duplicates_report(&raw) {
            return serde_json::json!({
                "schema_version": "bijux.fastq_stage_metrics.v1",
                "stage": "fastq.remove_duplicates",
                "tool": report.tool_id,
                "paired_mode": report.paired_mode,
                "threads": report.threads,
                "dedup_mode": report.dedup_mode,
                "keep_order": report.keep_order,
                "reads_in": report.reads_in,
                "reads_out": report.reads_out,
                "input_reads": report.reads_in,
                "duplicate_reads": report.duplicates_removed,
                "unique_reads": report.reads_out,
                "output_reads": report.reads_out,
                "reads_in_r2": report.reads_in_r2,
                "reads_out_r2": report.reads_out_r2,
                "pairs_in": report.pairs_in,
                "pairs_out": report.pairs_out,
                "pair_count_match": report.pair_count_match,
                "duplicates_removed": report.duplicates_removed,
                "dedup_rate": report.dedup_rate,
                "duplicate_class_count": report.duplicate_classes.len(),
                "duplicate_classes_tsv": report.duplicate_classes_tsv,
                "duplicate_provenance_json": report.duplicate_provenance_json,
                "raw_backend_report": report.raw_backend_report,
                "raw_backend_report_format": report.raw_backend_report_format,
                "report_json": report_path,
            });
        }
    }
    serde_json::json!({
        "schema_version": "bijux.fastq_stage_metrics.v1",
        "stage": "fastq.remove_duplicates",
        "tool": "report_missing",
        "input_reads": serde_json::Value::Null,
        "duplicate_reads": serde_json::Value::Null,
        "unique_reads": serde_json::Value::Null,
        "output_reads": serde_json::Value::Null,
        "duplicates_removed": serde_json::Value::Null,
        "report_json": report_path,
    })
}

pub(crate) fn parse_trim_polyg_metrics(out_dir: &std::path::Path) -> serde_json::Value {
    let report_path = out_dir.join("trim_polyg_tails_report.json");
    if let Ok(raw) = std::fs::read_to_string(&report_path) {
        if let Ok(report) = bijux_dna_domain_fastq::observer::parse_trim_polyg_report(&raw) {
            let reads_retained = report.reads_out;
            let reads_dropped = report
                .reads_in
                .zip(report.reads_out)
                .map(|(reads_in, reads_out)| reads_in.saturating_sub(reads_out));
            let bases_removed = report
                .bases_in
                .zip(report.bases_out)
                .map(|(bases_in, bases_out)| bases_in.saturating_sub(bases_out));
            return serde_json::json!({
                "schema_version": "bijux.fastq_stage_metrics.v1",
                "stage": "fastq.trim_polyg_tails",
                "tool": report.tool_id,
                "paired_mode": report.paired_mode,
                "threads": report.threads,
                "trim_polyg": report.trim_polyg,
                "min_polyg_run": report.min_polyg_run,
                "reads_in": report.reads_in,
                "reads_out": report.reads_out,
                "reads_retained": reads_retained,
                "reads_dropped": reads_dropped,
                "bases_in": report.bases_in,
                "bases_out": report.bases_out,
                "bases_removed": bases_removed,
                "pairs_in": report.pairs_in,
                "pairs_out": report.pairs_out,
                "mean_q_before": report.mean_q_before,
                "mean_q_after": report.mean_q_after,
                "trimmed_tail_count": report.trimmed_tail_count,
                "bases_trimmed_polyg": report.bases_trimmed_polyg,
                "polyx_bank_id": report.polyx_bank_id,
                "polyx_bank_hash": report.polyx_bank_hash,
                "polyx_preset": report.polyx_preset,
                "runtime_s": report.runtime_s,
                "memory_mb": report.memory_mb,
                "raw_backend_report_format": report.raw_backend_report_format,
                "report_json": report_path,
            });
        }
    }
    serde_json::json!({
        "schema_version": "bijux.fastq_stage_metrics.v1",
        "stage": "fastq.trim_polyg_tails",
        "tool": "report_missing",
        "trim_polyg": serde_json::Value::Null,
        "reads_in": serde_json::Value::Null,
        "reads_out": serde_json::Value::Null,
        "reads_retained": serde_json::Value::Null,
        "reads_dropped": serde_json::Value::Null,
        "bases_removed": serde_json::Value::Null,
        "report_json": report_path,
    })
}

pub(crate) fn parse_normalize_primers_metrics(out_dir: &std::path::Path) -> serde_json::Value {
    let report_path = out_dir.join("normalize_primers_report.json");
    if let Ok(raw) = std::fs::read_to_string(&report_path) {
        if let Ok(report) = bijux_dna_domain_fastq::observer::parse_normalize_primers_report(&raw) {
            let unmatched_reads = report
                .reads_in
                .zip(report.primer_trimmed_reads)
                .map(|(reads_in, matched_primers)| reads_in.saturating_sub(matched_primers));
            let trimmed_primer_bases = report
                .bases_in
                .zip(report.bases_out)
                .map(|(bases_in, bases_out)| bases_in.saturating_sub(bases_out));
            return serde_json::json!({
                "schema_version": "bijux.fastq_stage_metrics.v1",
                "stage": "fastq.normalize_primers",
                "tool": report.tool_id,
                "paired_mode": report.paired_mode,
                "primer_set_id": report.primer_set_id,
                "marker_id": report.marker_id,
                "orientation_policy": report.orientation_policy,
                "max_mismatch_rate": report.max_mismatch_rate,
                "min_overlap_bp": report.min_overlap_bp,
                "normalized_reads_r1": report.output_r1,
                "normalized_reads_r2": report.output_r2,
                "reads_in": report.reads_in,
                "reads_out": report.reads_out,
                "pairs_in": report.pairs_in,
                "pairs_out": report.pairs_out,
                "matched_primers": report.primer_trimmed_reads,
                "unmatched_reads": unmatched_reads,
                "trimmed_primer_bases": trimmed_primer_bases,
                "primer_trimmed_reads": report.primer_trimmed_reads,
                "primer_trimmed_fraction": report.primer_trimmed_fraction,
                "orientation_forward_fraction": report.orientation_forward_fraction,
                "primer_orientation_report": report.primer_orientation_report,
                "primer_stats_json": report.primer_stats_json,
                "raw_backend_report_format": report.raw_backend_report_format,
                "report_json": report_path,
            });
        }
    }
    serde_json::json!({
        "schema_version": "bijux.fastq_stage_metrics.v1",
        "stage": "fastq.normalize_primers",
        "tool": "report_missing",
        "matched_primers": serde_json::Value::Null,
        "unmatched_reads": serde_json::Value::Null,
        "trimmed_primer_bases": serde_json::Value::Null,
        "primer_trimmed_fraction": serde_json::Value::Null,
        "orientation_forward_fraction": serde_json::Value::Null,
        "report_json": report_path,
    })
}

pub(crate) fn parse_normalize_abundance_metrics(out_dir: &std::path::Path) -> serde_json::Value {
    let report_path = out_dir.join("normalize_abundance_report.json");
    if let Ok(raw) = std::fs::read_to_string(&report_path) {
        if let Ok(report) = bijux_dna_domain_fastq::observer::parse_normalize_abundance_report(&raw)
        {
            let expected_sum = report.scale_factor.unwrap_or(1.0);
            let numeric_output_valid = report
                .per_sample_sums
                .iter()
                .all(|(_, sum)| sum.is_finite() && (sum - expected_sum).abs() <= 1.0e-6);
            return serde_json::json!({
                "schema_version": "bijux.fastq_stage_metrics.v1",
                "stage": "fastq.normalize_abundance",
                "tool": report.tool_id,
                "method": report.method,
                "normalization_method": report.method,
                "normalized_abundance_tsv": report.normalized_abundance_tsv,
                "input_value_column": report.input_value_column,
                "normalized_value_column": report.normalized_value_column,
                "compositional_rule": report.compositional_rule,
                "scale_factor": report.scale_factor,
                "table_rows": report.table_rows,
                "sample_count": report.sample_count,
                "feature_count": report.feature_count,
                "zero_fraction": report.zero_fraction,
                "sample_totals": report.per_sample_sums,
                "per_sample_sum_count": report.per_sample_sums.len(),
                "numeric_output_valid": numeric_output_valid,
                "used_fallback": report.used_fallback,
                "raw_backend_report_format": report.raw_backend_report_format,
                "report_json": report_path,
            });
        }
    }
    serde_json::json!({
        "schema_version": "bijux.fastq_stage_metrics.v1",
        "stage": "fastq.normalize_abundance",
        "tool": "report_missing",
        "method": serde_json::Value::Null,
        "normalization_method": serde_json::Value::Null,
        "normalized_abundance_tsv": serde_json::Value::Null,
        "table_rows": serde_json::Value::Null,
        "sample_count": serde_json::Value::Null,
        "sample_totals": serde_json::Value::Null,
        "numeric_output_valid": serde_json::Value::Null,
        "zero_fraction": serde_json::Value::Null,
        "report_json": report_path,
    })
}

pub(crate) fn parse_remove_chimeras_metrics(out_dir: &std::path::Path) -> serde_json::Value {
    let report_path = out_dir.join("remove_chimeras_report.json");
    if let Ok(raw) = std::fs::read_to_string(&report_path) {
        if let Ok(report) = bijux_dna_domain_fastq::observer::parse_remove_chimeras_report(&raw) {
            return serde_json::json!({
                "schema_version": "bijux.fastq_stage_metrics.v1",
                "stage": "fastq.remove_chimeras",
                "tool": report.tool_id,
                "paired_mode": report.paired_mode,
                "method": report.method,
                "detection_scope": report.detection_scope,
                "filtered_representative_sequences": report.output_reads,
                "reads_in": report.reads_in,
                "reads_out": report.reads_out,
                "chimeras_removed": report.chimeras_removed,
                "chimera_count": report.chimeras_removed,
                "non_chimera_count": report.reads_out,
                "chimera_fraction": report.chimera_fraction,
                "used_fallback": report.used_fallback,
                "raw_backend_report": report.raw_backend_report,
                "raw_backend_report_format": report.raw_backend_report_format,
                "report_json": report_path,
            });
        }
    }
    serde_json::json!({
        "schema_version": "bijux.fastq_stage_metrics.v1",
        "stage": "fastq.remove_chimeras",
        "tool": "report_missing",
        "filtered_representative_sequences": serde_json::Value::Null,
        "chimera_count": serde_json::Value::Null,
        "non_chimera_count": serde_json::Value::Null,
        "chimera_fraction": serde_json::Value::Null,
        "chimeras_removed": serde_json::Value::Null,
        "raw_backend_report": serde_json::Value::Null,
        "report_json": report_path,
    })
}

pub(crate) fn parse_screen_taxonomy_metrics(out_dir: &std::path::Path) -> serde_json::Value {
    let report_path = discover_screen_taxonomy_report(out_dir)
        .unwrap_or_else(|| out_dir.join("classification_report.json"));
    if let Ok(raw) = std::fs::read_to_string(&report_path) {
        if let Ok(report) = bijux_dna_domain_fastq::observer::parse_screen_taxonomy_report(&raw) {
            let (classified_reads, unclassified_reads) =
                derive_screen_taxonomy_read_counts(&report);
            return serde_json::json!({
                "schema_version": "bijux.fastq_stage_metrics.v1",
                "stage": "fastq.screen_taxonomy",
                "tool": report.tool_id,
                "paired_mode": report.paired_mode,
                "classifier": report.classifier,
                "taxonomy_database_id": report.database_artifact_id,
                "report_format": report.report_format,
                "assignment_format": report.assignment_format,
                "database_catalog_id": report.database_catalog_id,
                "database_artifact_id": report.database_artifact_id,
                "database_build_id": report.database_build_id,
                "database_digest": report.database_digest,
                "database_namespace": report.database_namespace,
                "database_scope": report.database_scope,
                "minimum_confidence": report.minimum_confidence,
                "emit_unclassified": report.emit_unclassified,
                "reads_in": report.reads_in,
                "reads_out": report.reads_out,
                "classified_reads": classified_reads,
                "unclassified_reads": unclassified_reads,
                "bases_in": report.bases_in,
                "bases_out": report.bases_out,
                "pairs_in": report.pairs_in,
                "pairs_out": report.pairs_out,
                "contamination_rate": report.contamination_rate,
                "classified_fraction": report.classified_fraction,
                "unclassified_fraction": report.unclassified_fraction,
                "summary_entries": report.summary_entries,
                "top_taxa": report.top_taxa,
                "report_json": report_path,
            });
        }
    }
    serde_json::json!({
        "schema_version": "bijux.fastq_stage_metrics.v1",
        "stage": "fastq.screen_taxonomy",
        "tool": "report_missing",
        "classifier": serde_json::Value::Null,
        "taxonomy_database_id": serde_json::Value::Null,
        "classified_reads": serde_json::Value::Null,
        "unclassified_reads": serde_json::Value::Null,
        "contamination_rate": serde_json::Value::Null,
        "top_taxa": serde_json::Value::Null,
        "report_json": report_path,
    })
}

fn derive_screen_taxonomy_read_counts(
    report: &bijux_dna_domain_fastq::ScreenTaxonomyReportV1,
) -> (Option<u64>, Option<u64>) {
    let total_reads = report.reads_in.or(report.reads_out);
    match (total_reads, report.unclassified_fraction, report.classified_fraction) {
        (Some(total_reads), Some(unclassified_fraction), _) => {
            let unclassified_reads = ((total_reads as f64) * unclassified_fraction)
                .round()
                .clamp(0.0, total_reads as f64) as u64;
            let classified_reads = total_reads.saturating_sub(unclassified_reads);
            (Some(classified_reads), Some(unclassified_reads))
        }
        (Some(total_reads), None, Some(classified_fraction)) => {
            let classified_reads = ((total_reads as f64) * classified_fraction)
                .round()
                .clamp(0.0, total_reads as f64) as u64;
            let unclassified_reads = total_reads.saturating_sub(classified_reads);
            (Some(classified_reads), Some(unclassified_reads))
        }
        _ => (None, None),
    }
}

fn discover_screen_taxonomy_report(out_dir: &std::path::Path) -> Option<std::path::PathBuf> {
    [
        "kraken2.classifications.json",
        "krakenuniq.classifications.json",
        "centrifuge.classifications.json",
        "kaiju.classifications.json",
        "classification_report.json",
    ]
    .into_iter()
    .map(|name| out_dir.join(name))
    .find(|path| path.exists())
}

pub(crate) fn parse_deplete_rrna_metrics(out_dir: &std::path::Path) -> serde_json::Value {
    let report_path = out_dir.join("rrna_report.json");
    if let Ok(raw) = std::fs::read_to_string(&report_path) {
        if let Ok(report) = bijux_dna_domain_fastq::observer::parse_deplete_rrna_report(&raw) {
            return serde_json::json!({
                "schema_version": "bijux.fastq_stage_metrics.v1",
                "stage": "fastq.deplete_rrna",
                "tool": report.tool_id,
                "paired_mode": report.paired_mode,
                "threads": report.threads,
                "rrna_db": report.rrna_db,
                "database_artifact_id": report.database_artifact_id,
                "database_build_id": report.database_build_id,
                "database_digest": report.database_digest,
                "screening_engine": report.screening_engine,
                "report_format": report.report_format,
                "emit_removed_reads": report.emit_removed_reads,
                "min_identity": report.min_identity,
                "retained_read_role": report.retained_read_role,
                "rejected_read_role": report.rejected_read_role,
                "reads_in": report.reads_in,
                "reads_out": report.reads_out,
                "retained_reads": report.reads_out,
                "reads_removed": report.reads_removed,
                "removed_reads": report.reads_removed,
                "bases_in": report.bases_in,
                "bases_out": report.bases_out,
                "bases_removed": report.bases_removed,
                "pairs_in": report.pairs_in,
                "pairs_out": report.pairs_out,
                "rrna_fraction_removed": report.rrna_fraction_removed,
                "depletion_rate": report.rrna_fraction_removed,
                "raw_backend_report": report.raw_backend_report,
                "raw_backend_report_format": report.raw_backend_report_format,
                "report_json": report_path,
            });
        }
    }
    serde_json::json!({
        "schema_version": "bijux.fastq_stage_metrics.v1",
        "stage": "fastq.deplete_rrna",
        "tool": "report_missing",
        "retained_reads": serde_json::Value::Null,
        "rrna_fraction_removed": serde_json::Value::Null,
        "depletion_rate": serde_json::Value::Null,
        "reads_removed": serde_json::Value::Null,
        "removed_reads": serde_json::Value::Null,
        "report_json": report_path,
    })
}

pub(crate) fn parse_deplete_reference_contaminants_metrics(
    out_dir: &std::path::Path,
) -> serde_json::Value {
    let report_path = out_dir.join("contaminant_screen_report.json");
    if let Ok(raw) = std::fs::read_to_string(&report_path) {
        if let Ok(report) =
            bijux_dna_domain_fastq::observer::parse_deplete_reference_contaminants_report(&raw)
        {
            return serde_json::json!({
                "schema_version": "bijux.fastq_stage_metrics.v1",
                "stage": "fastq.deplete_reference_contaminants",
                "tool": report.tool_id,
                "paired_mode": report.paired_mode,
                "threads": report.threads,
                "reference_catalog_id": report.reference_catalog_id,
                "contaminant_reference": report.contaminant_reference,
                "reference_index_artifact_id": report.reference_index_artifact_id,
                "contaminant_index_artifact_id": report.reference_index_artifact_id,
                "reference_index_backend": report.reference_index_backend,
                "reference_build_id": report.reference_build_id,
                "reference_digest": report.reference_digest,
                "match_threshold": report.match_threshold,
                "retained_read_role": report.retained_read_role,
                "rejected_read_role": report.rejected_read_role,
                "retain_unmapped_pairs": report.retain_unmapped_pairs,
                "contaminant_screened_reads_r1": report.output_r1,
                "contaminant_screened_reads_r2": report.output_r2,
                "reads_in": report.reads_in,
                "reads_out": report.reads_out,
                "reads_removed": report.reads_removed,
                "contaminant_reads": report.reads_removed,
                "bases_in": report.bases_in,
                "bases_out": report.bases_out,
                "bases_removed": report.bases_removed,
                "pairs_in": report.pairs_in,
                "pairs_out": report.pairs_out,
                "contaminant_fraction_removed": report.contaminant_fraction_removed,
                "contaminant_hit_rate": report.contaminant_fraction_removed,
                "raw_backend_report": report.raw_backend_report,
                "raw_backend_report_format": report.raw_backend_report_format,
                "report_json": report_path,
            });
        }
    }
    serde_json::json!({
        "schema_version": "bijux.fastq_stage_metrics.v1",
        "stage": "fastq.deplete_reference_contaminants",
        "tool": "report_missing",
        "contaminant_index_artifact_id": serde_json::Value::Null,
        "contaminant_screened_reads_r1": serde_json::Value::Null,
        "contaminant_reads": serde_json::Value::Null,
        "contaminant_fraction_removed": serde_json::Value::Null,
        "contaminant_hit_rate": serde_json::Value::Null,
        "reads_removed": serde_json::Value::Null,
        "report_json": report_path,
    })
}

pub(crate) fn parse_deplete_host_metrics(out_dir: &std::path::Path) -> serde_json::Value {
    let report_path = out_dir.join("host_depletion_report.json");
    if let Ok(raw) = std::fs::read_to_string(&report_path) {
        if let Ok(report) = bijux_dna_domain_fastq::observer::parse_deplete_host_report(&raw) {
            return serde_json::json!({
                "schema_version": "bijux.fastq_stage_metrics.v1",
                "stage": "fastq.deplete_host",
                "tool": report.tool_id,
                "paired_mode": report.paired_mode,
                "threads": report.threads,
                "reference_scope": report.reference_scope,
                "reference_catalog_id": report.reference_catalog_id,
                "reference_index_artifact_id": report.reference_index_artifact_id,
                "host_index_artifact_id": report.reference_index_artifact_id,
                "reference_index_backend": report.reference_index_backend,
                "reference_build_id": report.reference_build_id,
                "reference_digest": report.reference_digest,
                "masking_policy": report.masking_policy,
                "decoy_policy": report.decoy_policy,
                "decoy_catalog_id": report.decoy_catalog_id,
                "identity_threshold": report.identity_threshold,
                "retained_read_policy": report.retained_read_policy,
                "emit_removed_reads": report.emit_removed_reads,
                "report_format": report.report_format,
                "retain_unmapped_pairs": report.retain_unmapped_pairs,
                "host_depleted_reads_r1": report.output_r1,
                "host_depleted_reads_r2": report.output_r2,
                "removed_host_reads_r1": report.removed_host_r1,
                "removed_host_reads_r2": report.removed_host_r2,
                "reads_in": report.reads_in,
                "reads_out": report.reads_out,
                "reads_removed": report.reads_removed,
                "depleted_reads": report.reads_removed,
                "bases_in": report.bases_in,
                "bases_out": report.bases_out,
                "bases_removed": report.bases_removed,
                "pairs_in": report.pairs_in,
                "pairs_out": report.pairs_out,
                "host_fraction_removed": report.host_fraction_removed,
                "host_hit_rate": report.host_fraction_removed,
                "raw_backend_report": report.raw_backend_report,
                "raw_backend_report_format": report.raw_backend_report_format,
                "report_json": report_path,
            });
        }
    }
    serde_json::json!({
        "schema_version": "bijux.fastq_stage_metrics.v1",
        "stage": "fastq.deplete_host",
        "tool": "report_missing",
        "host_index_artifact_id": serde_json::Value::Null,
        "host_depleted_reads_r1": serde_json::Value::Null,
        "depleted_reads": serde_json::Value::Null,
        "host_fraction_removed": serde_json::Value::Null,
        "host_hit_rate": serde_json::Value::Null,
        "reads_removed": serde_json::Value::Null,
        "report_json": report_path,
    })
}

pub(crate) fn parse_report_qc_metrics(out_dir: &std::path::Path) -> serde_json::Value {
    let report_path = out_dir.join("report_qc_report.json");
    if let Ok(raw) = std::fs::read_to_string(&report_path) {
        if let Ok(report) = bijux_dna_domain_fastq::observer::parse_report_qc_report(&raw) {
            return serde_json::json!({
                "schema_version": "bijux.fastq_stage_metrics.v1",
                "stage": "fastq.report_qc",
                "tool": report.tool_id,
                "aggregation_engine": report.aggregation_engine,
                "aggregation_scope": report.aggregation_scope,
                "reads_in": report.reads_in,
                "reads_out": report.reads_out,
                "bases_in": report.bases_in,
                "bases_out": report.bases_out,
                "pairs_in": report.pairs_in,
                "pairs_out": report.pairs_out,
                "mean_q": report.mean_q,
                "contamination_rate": report.contamination_rate,
                "adapter_content_max": report.adapter_content_max,
                "adapter_content_mean": report.adapter_content_mean,
                "duplication_rate": report.duplication_rate,
                "n_rate": report.n_rate,
                "kmer_warning_count": report.kmer_warning_count,
                "overrepresented_sequence_count": report.overrepresented_sequence_count,
                "governed_qc_input_count": report.governed_qc_input_count,
                "governed_qc_contributor_stage_ids": report.governed_qc_contributor_stage_ids,
                "governed_qc_contributor_tool_ids": report.governed_qc_contributor_tool_ids,
                "governed_qc_contributors": report.governed_qc_contributors,
                "governed_qc_lineage_hash": report.governed_qc_lineage_hash,
                "multiqc_sample_count": report.multiqc_sample_count,
                "multiqc_module_count": report.multiqc_module_count,
                "raw_fastqc_dir": report.raw_fastqc_dir,
                "trimmed_fastqc_dir": report.trimmed_fastqc_dir,
                "multiqc_report": report.multiqc_report,
                "multiqc_data": report.multiqc_data,
                "governed_qc_inputs_manifest": report.governed_qc_inputs_manifest,
                "report_json": report_path,
            });
        }
    }
    serde_json::json!({
        "schema_version": "bijux.fastq_stage_metrics.v1",
        "stage": "fastq.report_qc",
        "tool": "report_missing",
        "aggregation_engine": serde_json::Value::Null,
        "aggregation_scope": serde_json::Value::Null,
        "multiqc_report": serde_json::Value::Null,
        "multiqc_data": serde_json::Value::Null,
        "report_json": report_path,
    })
}

pub(crate) fn parse_detect_adapters_metrics(out_dir: &std::path::Path) -> serde_json::Value {
    let report_path = out_dir.join("adapter_report.json");
    if let Ok(raw) = std::fs::read_to_string(&report_path) {
        if let Ok(report) = bijux_dna_domain_fastq::observer::parse_detect_adapters_report(&raw) {
            return serde_json::json!({
                "schema_version": "bijux.fastq_stage_metrics.v1",
                "stage": "fastq.detect_adapters",
                "tool": report.tool_id,
                "paired_mode": report.paired_mode,
                "threads": report.threads,
                "inspection_mode": report.inspection_mode,
                "report_only": report.report_only,
                "evidence_engine": report.evidence_engine,
                "evidence_scope": report.evidence_scope,
                "evidence_format": report.evidence_format,
                "adapter_report": report.report_json,
                "candidate_adapter_count": report.candidate_adapter_count,
                "detected_adapter_ids": report.detected_adapter_ids,
                "detection_confidence": report.detection_confidence,
                "detection_threshold": report.detection_threshold,
                "adapter_trimmed_fraction": report.adapter_trimmed_fraction,
                "adapter_content_max": report.adapter_content_max,
                "adapter_content_mean": report.adapter_content_mean,
                "duplication_rate": report.duplication_rate,
                "n_rate": report.n_rate,
                "kmer_warning_count": report.kmer_warning_count,
                "overrepresented_sequence_count": report.overrepresented_sequence_count,
                "adapter_inference": {
                    "source": report.evidence_engine,
                    "adapter_report": report.report_json,
                    "candidate_adapter_count": report.candidate_adapter_count,
                    "detected_adapter_ids": report.detected_adapter_ids,
                    "detection_confidence": report.detection_confidence,
                    "detection_threshold": report.detection_threshold,
                    "adapter_trimmed_fraction": report.adapter_trimmed_fraction,
                    "adapter_evidence_dir": report.adapter_evidence_dir,
                },
                "report_json": report_path,
            });
        }
    }
    serde_json::json!({
        "schema_version": "bijux.fastq_stage_metrics.v1",
        "stage": "fastq.detect_adapters",
        "tool": "report_missing",
        "adapter_report": serde_json::Value::Null,
        "candidate_adapter_count": serde_json::Value::Null,
        "detected_adapter_ids": serde_json::Value::Null,
        "detection_confidence": serde_json::Value::Null,
        "detection_threshold": serde_json::Value::Null,
        "adapter_inference": {
            "detected": out_dir.join("fastqc").exists(),
            "source": "report_missing",
            "output_dir": out_dir.join("fastqc"),
        },
        "report_json": report_path,
    })
}

pub(crate) fn parse_index_reference_metrics(out_dir: &std::path::Path) -> serde_json::Value {
    let report_path = out_dir.join("index_reference_report.json");
    if let Ok(raw) = std::fs::read_to_string(&report_path) {
        if let Ok(report) = bijux_dna_domain_fastq::observer::parse_index_reference_report(&raw) {
            let index_directory = report
                .backend_metrics
                .as_ref()
                .and_then(|metrics| metrics.get("index_directory"))
                .and_then(serde_json::Value::as_str)
                .map(std::string::ToString::to_string)
                .or_else(|| {
                    std::path::Path::new(&report.reference_index)
                        .parent()
                        .map(|path| path.to_string_lossy().to_string())
                });
            return serde_json::json!({
                "schema_version": "bijux.fastq_stage_metrics.v1",
                "stage": "fastq.index_reference",
                "tool": report.tool_id,
                "threads": report.threads,
                "index_format": report.index_format,
                "index_directory": index_directory,
                "index_files": report.emitted_files,
                "elapsed_time_s": report.runtime_s,
                "index_size_bytes": report.index_bytes,
                "reference_bytes": report.reference_bytes,
                "index_bytes": report.index_bytes,
                "index_file_count": report.index_file_count,
                "index_prefix": report.index_prefix,
                "emitted_file_count": report.emitted_files.len(),
                "report_json": report_path,
            });
        }
    }
    serde_json::json!({
        "schema_version": "bijux.fastq_stage_metrics.v1",
        "stage": "fastq.index_reference",
        "tool": "report_missing",
        "index_directory": serde_json::Value::Null,
        "index_files": [],
        "elapsed_time_s": serde_json::Value::Null,
        "index_size_bytes": serde_json::Value::Null,
        "reference_bytes": serde_json::Value::Null,
        "index_bytes": serde_json::Value::Null,
        "index_file_count": serde_json::Value::Null,
        "report_json": report_path,
    })
}
