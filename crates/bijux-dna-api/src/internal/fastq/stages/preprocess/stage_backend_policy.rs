pub(super) fn canonical_sample_identity(sample_id: &str) -> String {
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

pub(super) fn parse_first_u64_after_key(text: &str, key: &str) -> Option<u64> {
    for line in text.lines() {
        if !line
            .to_ascii_lowercase()
            .contains(&key.to_ascii_lowercase())
        {
            continue;
        }
        let digits: String = line.chars().filter(char::is_ascii_digit).collect();
        if let Ok(parsed) = digits.parse::<u64>() {
            return Some(parsed);
        }
    }
    None
}

pub(super) fn parse_validate_reads_metrics(
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

pub(super) fn parse_profile_reads_metrics(out_dir: &std::path::Path) -> serde_json::Value {
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

pub(super) fn parse_filter_reads_metrics(out_dir: &std::path::Path) -> serde_json::Value {
    let report_path = out_dir.join("filter_report.json");
    if let Ok(raw) = std::fs::read_to_string(&report_path) {
        if let Ok(report) = bijux_dna_domain_fastq::observer::parse_filter_reads_report(&raw) {
            return serde_json::json!({
                "schema_version": "bijux.fastq_stage_metrics.v1",
                "stage": "fastq.filter_reads",
                "tool": report.tool_id,
                "paired_mode": report.paired_mode,
                "threads": report.threads,
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
        "reads_in": serde_json::Value::Null,
        "reads_out": serde_json::Value::Null,
        "reads_dropped": serde_json::Value::Null,
        "report_json": report_path,
    })
}

pub(super) fn parse_correct_errors_metrics(out_dir: &std::path::Path) -> serde_json::Value {
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
                "corrected_reads": report.corrected_reads,
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
        "correction_engine": serde_json::Value::Null,
        "corrected_reads": serde_json::Value::Null,
        "kmer_fix_rate": serde_json::Value::Null,
        "report_json": report_path,
    })
}

pub(super) fn parse_filter_low_complexity_metrics(out_dir: &std::path::Path) -> serde_json::Value {
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
                "entropy_threshold": report.entropy_threshold,
                "polyx_threshold": report.polyx_threshold,
                "reads_in": report.reads_in,
                "reads_out": report.reads_out,
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
        "reads_removed_low_complexity": serde_json::Value::Null,
        "report_json": report_path,
    })
}

pub(super) fn parse_profile_read_lengths_metrics(out_dir: &std::path::Path) -> serde_json::Value {
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
                "mean_read_length": report.mean_read_length,
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
        "mean_read_length": serde_json::Value::Null,
        "report_json": report_path,
    })
}

pub(super) fn parse_profile_overrepresented_metrics(
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

pub(super) fn parse_infer_asvs_metrics(out_dir: &std::path::Path) -> serde_json::Value {
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
        "asv_count": serde_json::Value::Null,
        "sample_count": serde_json::Value::Null,
        "representative_sequence_count": serde_json::Value::Null,
        "report_json": report_path,
    })
}

pub(super) fn parse_extract_umis_metrics(out_dir: &std::path::Path) -> serde_json::Value {
    let report_path = out_dir.join("umi_report.json");
    if let Ok(raw) = std::fs::read_to_string(&report_path) {
        if let Ok(report) = bijux_dna_domain_fastq::observer::parse_extract_umis_report(&raw) {
            return serde_json::json!({
                "schema_version": "bijux.fastq_stage_metrics.v1",
                "stage": "fastq.extract_umis",
                "tool": report.tool_id,
                "paired_mode": report.paired_mode,
                "threads": report.threads,
                "umi_pattern": report.umi_pattern,
                "reads_in": report.reads_in,
                "reads_out": report.reads_out,
                "bases_in": report.bases_in,
                "bases_out": report.bases_out,
                "pairs_in": report.pairs_in,
                "pairs_out": report.pairs_out,
                "reads_with_umi": report.reads_with_umi,
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
        "reads_in": serde_json::Value::Null,
        "reads_out": serde_json::Value::Null,
        "report_json": report_path,
    })
}

pub(super) fn parse_trim_terminal_damage_metrics(out_dir: &std::path::Path) -> serde_json::Value {
    let report_path = out_dir.join("trim_terminal_damage_report.json");
    if let Ok(raw) = std::fs::read_to_string(&report_path) {
        if let Ok(report) = bijux_dna_domain_fastq::observer::parse_terminal_damage_report(&raw) {
            return serde_json::json!({
                "schema_version": "bijux.fastq_stage_metrics.v1",
                "stage": "fastq.trim_terminal_damage",
                "tool": report.tool_id,
                "paired_mode": report.paired_mode,
                "threads": report.threads,
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
        "udg_classification": serde_json::Value::Null,
        "ct_ga_asymmetry_pre": serde_json::Value::Null,
        "ct_ga_asymmetry_post": serde_json::Value::Null,
        "report_json": report_path,
    })
}

pub(super) fn parse_trim_reads_metrics(out_dir: &std::path::Path) -> serde_json::Value {
    let report_path = out_dir.join("trim_report.json");
    if let Ok(raw) = std::fs::read_to_string(&report_path) {
        if let Ok(report) = bijux_dna_domain_fastq::observer::parse_trim_reads_report(&raw) {
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
                "reads_in": report.reads_in,
                "reads_out": report.reads_out,
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
        "stage": "fastq.trim_reads",
        "tool": "report_missing",
        "report_json": report_path,
    })
}

pub(super) fn parse_merge_pairs_metrics(out_dir: &std::path::Path) -> serde_json::Value {
    let report_path = out_dir.join("merge_report.json");
    if let Ok(raw) = std::fs::read_to_string(&report_path) {
        if let Ok(report) = bijux_dna_domain_fastq::observer::parse_merge_pairs_report(&raw) {
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
                "reads_merged": report.reads_merged,
                "reads_unmerged": report.reads_unmerged,
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
        "reads_merged": serde_json::Value::Null,
        "reads_unmerged": serde_json::Value::Null,
        "merge_rate": serde_json::Value::Null,
        "runtime_s": serde_json::Value::Null,
        "memory_mb": serde_json::Value::Null,
        "raw_backend_report": serde_json::Value::Null,
        "raw_backend_report_format": serde_json::Value::Null,
        "report_json": report_path,
    })
}

pub(super) fn parse_cluster_otus_metrics(out_dir: &std::path::Path) -> serde_json::Value {
    let report_path = out_dir.join("cluster_otus_report.json");
    if let Ok(raw) = std::fs::read_to_string(&report_path) {
        if let Ok(report) = bijux_dna_domain_fastq::observer::parse_cluster_otus_report(&raw) {
            return serde_json::json!({
                "schema_version": "bijux.fastq_stage_metrics.v1",
                "stage": "fastq.cluster_otus",
                "tool": report.tool_id,
                "otu_identity": report.otu_identity,
                "threads": report.threads,
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
        "otu_count": serde_json::Value::Null,
        "sample_count": serde_json::Value::Null,
        "representative_sequence_count": serde_json::Value::Null,
        "report_json": report_path,
    })
}

pub(super) fn parse_remove_duplicates_metrics(out_dir: &std::path::Path) -> serde_json::Value {
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
        "duplicates_removed": serde_json::Value::Null,
        "report_json": report_path,
    })
}

pub(super) fn parse_trim_polyg_metrics(out_dir: &std::path::Path) -> serde_json::Value {
    let report_path = out_dir.join("trim_polyg_tails_report.json");
    if let Ok(raw) = std::fs::read_to_string(&report_path) {
        if let Ok(report) = bijux_dna_domain_fastq::observer::parse_trim_polyg_report(&raw) {
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
                "bases_in": report.bases_in,
                "bases_out": report.bases_out,
                "pairs_in": report.pairs_in,
                "pairs_out": report.pairs_out,
                "mean_q_before": report.mean_q_before,
                "mean_q_after": report.mean_q_after,
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
        "report_json": report_path,
    })
}

pub(super) fn parse_normalize_primers_metrics(out_dir: &std::path::Path) -> serde_json::Value {
    let report_path = out_dir.join("normalize_primers_report.json");
    if let Ok(raw) = std::fs::read_to_string(&report_path) {
        if let Ok(report) = bijux_dna_domain_fastq::observer::parse_normalize_primers_report(&raw) {
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
                "reads_in": report.reads_in,
                "reads_out": report.reads_out,
                "pairs_in": report.pairs_in,
                "pairs_out": report.pairs_out,
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
        "primer_trimmed_fraction": serde_json::Value::Null,
        "orientation_forward_fraction": serde_json::Value::Null,
        "report_json": report_path,
    })
}

pub(super) fn parse_normalize_abundance_metrics(out_dir: &std::path::Path) -> serde_json::Value {
    let report_path = out_dir.join("normalize_abundance_report.json");
    if let Ok(raw) = std::fs::read_to_string(&report_path) {
        if let Ok(report) = bijux_dna_domain_fastq::observer::parse_normalize_abundance_report(&raw)
        {
            return serde_json::json!({
                "schema_version": "bijux.fastq_stage_metrics.v1",
                "stage": "fastq.normalize_abundance",
                "tool": report.tool_id,
                "method": report.method,
                "input_value_column": report.input_value_column,
                "normalized_value_column": report.normalized_value_column,
                "compositional_rule": report.compositional_rule,
                "scale_factor": report.scale_factor,
                "table_rows": report.table_rows,
                "sample_count": report.sample_count,
                "feature_count": report.feature_count,
                "zero_fraction": report.zero_fraction,
                "per_sample_sum_count": report.per_sample_sums.len(),
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
        "table_rows": serde_json::Value::Null,
        "sample_count": serde_json::Value::Null,
        "zero_fraction": serde_json::Value::Null,
        "report_json": report_path,
    })
}

pub(super) fn parse_remove_chimeras_metrics(out_dir: &std::path::Path) -> serde_json::Value {
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
                "reads_in": report.reads_in,
                "reads_out": report.reads_out,
                "chimeras_removed": report.chimeras_removed,
                "chimera_fraction": report.chimera_fraction,
                "used_fallback": report.used_fallback,
                "raw_backend_report_format": report.raw_backend_report_format,
                "report_json": report_path,
            });
        }
    }
    serde_json::json!({
        "schema_version": "bijux.fastq_stage_metrics.v1",
        "stage": "fastq.remove_chimeras",
        "tool": "report_missing",
        "chimera_fraction": serde_json::Value::Null,
        "chimeras_removed": serde_json::Value::Null,
        "report_json": report_path,
    })
}

pub(super) fn parse_screen_taxonomy_metrics(out_dir: &std::path::Path) -> serde_json::Value {
    let report_path = discover_screen_taxonomy_report(out_dir)
        .unwrap_or_else(|| out_dir.join("classification_report.json"));
    if let Ok(raw) = std::fs::read_to_string(&report_path) {
        if let Ok(report) = bijux_dna_domain_fastq::observer::parse_screen_taxonomy_report(&raw) {
            return serde_json::json!({
                "schema_version": "bijux.fastq_stage_metrics.v1",
                "stage": "fastq.screen_taxonomy",
                "tool": report.tool_id,
                "paired_mode": report.paired_mode,
                "classifier": report.classifier,
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
        "contamination_rate": serde_json::Value::Null,
        "top_taxa": serde_json::Value::Null,
        "report_json": report_path,
    })
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

pub(super) fn parse_deplete_rrna_metrics(out_dir: &std::path::Path) -> serde_json::Value {
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
                "screening_engine": report.screening_engine,
                "report_format": report.report_format,
                "emit_removed_reads": report.emit_removed_reads,
                "min_identity": report.min_identity,
                "reads_in": report.reads_in,
                "reads_out": report.reads_out,
                "reads_removed": report.reads_removed,
                "bases_in": report.bases_in,
                "bases_out": report.bases_out,
                "bases_removed": report.bases_removed,
                "pairs_in": report.pairs_in,
                "pairs_out": report.pairs_out,
                "rrna_fraction_removed": report.rrna_fraction_removed,
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
        "rrna_fraction_removed": serde_json::Value::Null,
        "reads_removed": serde_json::Value::Null,
        "report_json": report_path,
    })
}

pub(super) fn parse_deplete_reference_contaminants_metrics(
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
                "index_artifact": report.index_artifact,
                "reference_index_backend": report.reference_index_backend,
                "reference_build_id": report.reference_build_id,
                "reference_digest": report.reference_digest,
                "retain_unmapped_pairs": report.retain_unmapped_pairs,
                "reads_in": report.reads_in,
                "reads_out": report.reads_out,
                "reads_removed": report.reads_removed,
                "bases_in": report.bases_in,
                "bases_out": report.bases_out,
                "bases_removed": report.bases_removed,
                "pairs_in": report.pairs_in,
                "pairs_out": report.pairs_out,
                "contaminant_fraction_removed": report.contaminant_fraction_removed,
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
        "contaminant_fraction_removed": serde_json::Value::Null,
        "reads_removed": serde_json::Value::Null,
        "report_json": report_path,
    })
}

pub(super) fn parse_deplete_host_metrics(out_dir: &std::path::Path) -> serde_json::Value {
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
                "reads_in": report.reads_in,
                "reads_out": report.reads_out,
                "reads_removed": report.reads_removed,
                "bases_in": report.bases_in,
                "bases_out": report.bases_out,
                "bases_removed": report.bases_removed,
                "pairs_in": report.pairs_in,
                "pairs_out": report.pairs_out,
                "host_fraction_removed": report.host_fraction_removed,
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
        "host_fraction_removed": serde_json::Value::Null,
        "reads_removed": serde_json::Value::Null,
        "report_json": report_path,
    })
}

pub(super) fn parse_report_qc_metrics(out_dir: &std::path::Path) -> serde_json::Value {
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

pub(super) fn parse_detect_adapters_metrics(out_dir: &std::path::Path) -> serde_json::Value {
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
                "candidate_adapter_count": report.candidate_adapter_count,
                "adapter_trimmed_fraction": report.adapter_trimmed_fraction,
                "adapter_content_max": report.adapter_content_max,
                "adapter_content_mean": report.adapter_content_mean,
                "duplication_rate": report.duplication_rate,
                "n_rate": report.n_rate,
                "kmer_warning_count": report.kmer_warning_count,
                "overrepresented_sequence_count": report.overrepresented_sequence_count,
                "adapter_inference": {
                    "source": report.evidence_engine,
                    "candidate_adapter_count": report.candidate_adapter_count,
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
        "candidate_adapter_count": serde_json::Value::Null,
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
            return serde_json::json!({
                "schema_version": "bijux.fastq_stage_metrics.v1",
                "stage": "fastq.index_reference",
                "tool": report.tool_id,
                "threads": report.threads,
                "index_format": report.index_format,
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
        "reference_bytes": serde_json::Value::Null,
        "index_bytes": serde_json::Value::Null,
        "index_file_count": serde_json::Value::Null,
        "report_json": report_path,
    })
}

pub(super) fn stage_network_policy(stage_id: &str) -> NetworkPolicy {
    match stage_id {
        "fastq.validate_reads"
        | "fastq.detect_adapters"
        | "fastq.trim_terminal_damage"
        | "fastq.trim_reads"
        | "fastq.merge_pairs"
        | "fastq.remove_duplicates"
        | "fastq.correct_errors"
        | "fastq.filter_reads"
        | "fastq.filter_low_complexity"
        | "fastq.trim_polyg_tails"
        | "fastq.screen_taxonomy" => NetworkPolicy::Forbid,
        _ => NetworkPolicy::Allow,
    }
}

fn fastq_backend_allowlist(stage_id: &str) -> Option<Vec<String>> {
    if !stage_id.starts_with("fastq.") {
        return None;
    }
    let tools = bijux_dna_planner_fastq::stage_api::allowed_tools_for_stage(
        &bijux_dna_core::ids::StageId::new(stage_id.to_string()),
    );
    Some(
        tools
            .into_iter()
            .map(|tool| tool.to_string())
            .collect::<Vec<_>>(),
    )
}

pub(super) fn enforce_fastq_backend_allowlist(stage_id: &str, tool_id: &str) -> Result<()> {
    let Some(allowed) = fastq_backend_allowlist(stage_id) else {
        return Ok(());
    };
    if allowed.iter().any(|allowed_tool| allowed_tool == tool_id) {
        return Ok(());
    }
    Err(anyhow!(
        "unsupported backend for {stage_id}: `{tool_id}` not in allowlist {}",
        allowed.join(",")
    ))
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use std::path::Path;
    use std::sync::{Mutex, OnceLock};

    use anyhow::Result;
    use bijux_dna_runner::step_runner::StageResultV1;

    use super::{
        fastq_backend_allowlist, parse_deplete_host_metrics,
        parse_deplete_reference_contaminants_metrics,
        parse_deplete_rrna_metrics, parse_filter_low_complexity_metrics, parse_filter_reads_metrics,
        parse_extract_umis_metrics,
        parse_index_reference_metrics,
        parse_merge_pairs_metrics, parse_normalize_abundance_metrics,
        parse_normalize_primers_metrics, parse_profile_overrepresented_metrics,
        parse_profile_read_lengths_metrics, parse_profile_reads_metrics,
        parse_remove_duplicates_metrics, parse_report_qc_metrics, parse_screen_taxonomy_metrics,
        parse_trim_polyg_metrics, parse_trim_reads_metrics, parse_trim_terminal_damage_metrics,
        parse_validate_reads_metrics, required_metrics_keys,
    };

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    struct EnvGuard {
        key: &'static str,
        value: Option<String>,
    }

    impl EnvGuard {
        fn capture(key: &'static str) -> Self {
            Self {
                key,
                value: std::env::var(key).ok(),
            }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            if let Some(value) = self.value.take() {
                std::env::set_var(self.key, value);
            } else {
                std::env::remove_var(self.key);
            }
        }
    }

    #[test]
    fn fastq_backend_allowlist_matches_planner_registry_selection() -> Result<()> {
        let _lock = env_lock().lock().expect("lock env mutation tests");
        let _include_guard = EnvGuard::capture("BIJUX_INCLUDE_EXPERIMENTAL_TOOLS");
        let _api_guard = EnvGuard::capture("BIJUX_EXPERIMENTAL_TOOLS");
        std::env::remove_var("BIJUX_INCLUDE_EXPERIMENTAL_TOOLS");
        std::env::remove_var("BIJUX_EXPERIMENTAL_TOOLS");

        let stages_dir = crate::support::repo_root::resolve_repo_root()?.join("domain/fastq/stages");
        for entry in std::fs::read_dir(&stages_dir)? {
            let path = entry?.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("yaml") {
                continue;
            }
            if path.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
                continue;
            }
            let raw = std::fs::read_to_string(&path)?;
            let Some(stage_id) = raw
                .lines()
                .find_map(|line| line.strip_prefix("stage_id: "))
                .map(|value| value.trim().trim_matches('"').to_string())
            else {
                continue;
            };
            let expected = bijux_dna_planner_fastq::stage_api::allowed_tools_for_stage(
                &bijux_dna_core::ids::StageId::new(stage_id.clone()),
            )
            .into_iter()
            .map(|tool| tool.to_string())
            .collect::<Vec<_>>();
            let actual = fastq_backend_allowlist(&stage_id).unwrap_or_default();
            assert_eq!(
                actual, expected,
                "fastq API backend allowlist drifted from planner registry selection for {stage_id}"
            );
        }
        Ok(())
    }

    #[test]
    fn index_reference_standardized_metrics_prefer_governed_report() {
        let temp = tempfile::tempdir().expect("tempdir");
        let report_path = temp.path().join("index_reference_report.json");
        bijux_dna_infra::atomic_write_json(
            &report_path,
            &serde_json::json!({
                "schema_version": "bijux.fastq.index_reference.report.v2",
                "stage": "fastq.index_reference",
                "stage_id": "fastq.index_reference",
                "tool_id": "bowtie2_build",
                "threads": 4,
                "index_format": "bowtie2_build",
                "reference_fasta": "reference.fa",
                "reference_bytes": 4096,
                "reference_index": "reference_index/bowtie2/reference",
                "report_json": "index_reference_report.json",
                "index_prefix": "reference",
                "emitted_files": [
                    {"relative_path": "bowtie2/reference.1.bt2", "bytes": 1024},
                    {"relative_path": "bowtie2/reference.2.bt2", "bytes": 2048}
                ],
                "index_file_count": 2,
                "index_bytes": 3072,
                "runtime_s": 1.5,
                "memory_mb": 96.0,
                "exit_code": 0,
                "backend_metrics": {"index_directory": "reference_index/bowtie2"}
            }),
        )
        .expect("write report");

        let metrics = parse_index_reference_metrics(temp.path());
        assert_eq!(metrics["tool"], serde_json::json!("bowtie2_build"));
        assert_eq!(metrics["index_bytes"], serde_json::json!(3072));
        assert_eq!(metrics["emitted_file_count"], serde_json::json!(2));
    }

    #[test]
    fn fastq_backend_allowlist_for_governed_stage_is_env_toggle_invariant() {
        let _lock = env_lock().lock().expect("lock env mutation tests");
        let _include_guard = EnvGuard::capture("BIJUX_INCLUDE_EXPERIMENTAL_TOOLS");
        let _api_guard = EnvGuard::capture("BIJUX_EXPERIMENTAL_TOOLS");
        std::env::remove_var("BIJUX_INCLUDE_EXPERIMENTAL_TOOLS");
        std::env::remove_var("BIJUX_EXPERIMENTAL_TOOLS");

        let governed = fastq_backend_allowlist("fastq.trim_reads").unwrap_or_default();
        assert!(
            governed.iter().any(|tool| tool == "prinseq"),
            "governed trim backend allowlist must include prinseq from execution support"
        );

        std::env::set_var("BIJUX_EXPERIMENTAL_TOOLS", "1");
        let experimental = fastq_backend_allowlist("fastq.trim_reads").unwrap_or_default();
        assert!(
            experimental == governed,
            "governed FASTQ API allowlists must remain stable across experimental registry toggles"
        );
    }

    #[test]
    fn report_qc_uses_stage_specific_metrics_policy() {
        assert_eq!(
            required_metrics_keys("fastq.report_qc"),
            &[
                "schema_version",
                "stage",
                "tool",
                "aggregation_engine",
                "aggregation_scope",
                "governed_qc_input_count",
                "multiqc_report",
                "multiqc_data",
                "report_json",
            ]
        );
    }

    #[test]
    fn merge_pairs_uses_governed_report_metrics_policy() {
        assert_eq!(
            required_metrics_keys("fastq.merge_pairs"),
            &[
                "schema_version",
                "stage",
                "tool",
                "paired_mode",
                "merge_engine",
                "threads",
                "merge_overlap",
                "min_length",
                "unmerged_read_policy",
                "reads_r1",
                "reads_r2",
                "reads_merged",
                "reads_unmerged",
                "merge_rate",
                "raw_backend_report_format",
            ]
        );
    }

    #[test]
    fn validate_reads_uses_governed_report_metrics_policy() {
        assert_eq!(
            required_metrics_keys("fastq.validate_reads"),
            &[
                "schema_version",
                "stage",
                "validator",
                "failure_class",
                "strict_pass",
                "exit_code",
            ]
        );
    }

    #[test]
    fn correct_errors_uses_governed_report_metrics_policy() {
        assert_eq!(
            required_metrics_keys("fastq.correct_errors"),
            &[
                "schema_version",
                "stage",
                "tool",
                "correction_engine",
                "corrected_reads",
                "kmer_fix_rate",
            ]
        );
    }

    #[test]
    fn profile_reads_uses_governed_report_metrics_policy() {
        assert_eq!(
            required_metrics_keys("fastq.profile_reads"),
            &[
                "schema_version",
                "stage",
                "tool",
                "reads_total",
                "bases_total",
                "length_histogram_bins",
            ]
        );
    }

    #[test]
    fn filter_reads_uses_governed_report_metrics_policy() {
        assert_eq!(
            required_metrics_keys("fastq.filter_reads"),
            &[
                "schema_version",
                "stage",
                "tool",
                "reads_in",
                "reads_out",
                "reads_dropped",
            ]
        );
    }

    #[test]
    fn profile_read_lengths_uses_governed_report_metrics_policy() {
        assert_eq!(
            required_metrics_keys("fastq.profile_read_lengths"),
            &[
                "schema_version",
                "stage",
                "tool",
                "read_count",
                "mean_read_length",
                "histogram_entry_count",
            ]
        );
    }

    #[test]
    fn profile_overrepresented_uses_governed_report_metrics_policy() {
        assert_eq!(
            required_metrics_keys("fastq.profile_overrepresented_sequences"),
            &[
                "schema_version",
                "stage",
                "tool",
                "sequence_count",
                "flagged_sequences",
                "top_fraction",
            ]
        );
    }

    #[test]
    fn normalize_abundance_uses_governed_report_metrics_policy() {
        assert_eq!(
            required_metrics_keys("fastq.normalize_abundance"),
            &[
                "schema_version",
                "stage",
                "tool",
                "method",
                "table_rows",
                "sample_count",
                "zero_fraction",
            ]
        );
    }

    #[test]
    fn parse_validate_reads_metrics_prefers_governed_validation_report() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let out_dir = temp.path();
        std::fs::write(
            out_dir.join("validation.json"),
            serde_json::json!({
                "schema_version": "bijux.fastq.validate.report.v1",
                "stage": "fastq.validate_reads",
                "stage_id": "fastq.validate_reads",
                "tool_id": "seqtk",
                "validation_mode": "strict",
                "pair_sync_policy": "require_header_sync",
                "input_r1": "reads_R1.fastq.gz",
                "input_r2": "reads_R2.fastq.gz",
                "validation_log_r1": "validation_r1.log",
                "validation_log_r2": "validation_r2.log",
                "validated_inputs": 2,
                "validated_reads_r1": 42,
                "validated_reads_r2": 41,
                "validated_pairs": 41,
                "status_r1": 0,
                "status_r2": 0,
                "pair_sync_checked": true,
                "pair_sync_pass": false,
                "pair_count_match": false,
                "failure_class": "pair_count_mismatch",
                "strict_pass": false,
                "exit_code": 96
            })
            .to_string(),
        )?;

        let metrics = parse_validate_reads_metrics(
            out_dir,
            &StageResultV1 {
                run_id: "run-1".to_string(),
                runtime_s: 1.0,
                memory_mb: 32.0,
                exit_code: 1,
                outputs: Vec::new(),
                metrics_path: None,
                stdout: "ignored".to_string(),
                stderr: "ignored".to_string(),
                command: "seqtk".to_string(),
            },
        );

        assert_eq!(metrics["validator"], serde_json::json!("seqtk"));
        assert_eq!(
            metrics["failure_class"],
            serde_json::json!("pair_count_mismatch")
        );
        assert_eq!(metrics["validated_reads_r1"], serde_json::json!(42));
        assert_eq!(metrics["validated_reads_r2"], serde_json::json!(41));
        assert_eq!(metrics["pair_sync_pass"], serde_json::json!(false));
        assert_eq!(metrics["exit_code"], serde_json::json!(96));
        assert_eq!(
            metrics["report_json"],
            serde_json::json!(Path::new(out_dir).join("validation.json"))
        );
        Ok(())
    }

    #[test]
    fn trim_terminal_damage_standardized_metrics_prefer_governed_report() {
        let temp = tempfile::tempdir().expect("tempdir");
        let report_path = temp.path().join("trim_terminal_damage_report.json");
        std::fs::write(
            &report_path,
            serde_json::json!({
                "schema_version": "bijux.fastq.trim_terminal_damage.report.v2",
                "stage": "fastq.trim_terminal_damage",
                "stage_id": "fastq.trim_terminal_damage",
                "tool_id": "cutadapt",
                "paired_mode": "single_end",
                "threads": 4,
                "damage_mode": "ancient",
                "execution_policy": "explicit_terminal_trim",
                "trim_5p_bases": 2,
                "trim_3p_bases": 2,
                "requested_trim_5p_bases": 2,
                "requested_trim_3p_bases": 2,
                "udg_classification": "non_udg",
                "input_r1": "reads.fastq.gz",
                "input_r2": null,
                "output_r1": "trimmed.fastq.gz",
                "output_r2": null,
                "reads_in": null,
                "reads_out": null,
                "bases_in": null,
                "bases_out": null,
                "mean_q_before": null,
                "mean_q_after": null,
                "ct_ga_asymmetry_pre": 0.42,
                "ct_ga_asymmetry_post": 0.11,
                "ct_ga_asymmetry_pre_r1": null,
                "ct_ga_asymmetry_post_r1": null,
                "ct_ga_asymmetry_pre_r2": null,
                "ct_ga_asymmetry_post_r2": null,
                "terminal_base_composition_pre_r1": null,
                "terminal_base_composition_post_r1": null,
                "terminal_base_composition_pre_r2": null,
                "terminal_base_composition_post_r2": null,
                "raw_backend_report": "cutadapt.raw.json",
                "raw_backend_report_format": "cutadapt_json",
                "runtime_s": null,
                "memory_mb": null,
                "used_fallback": false,
                "backend_metrics": {"reads_profiled_r1": 100}
            })
            .to_string(),
        )
        .expect("write report");

        let metrics = parse_trim_terminal_damage_metrics(temp.path());
        assert_eq!(metrics["tool"], serde_json::json!("cutadapt"));
        assert_eq!(
            metrics["execution_policy"],
            serde_json::json!("explicit_terminal_trim")
        );
        assert_eq!(metrics["threads"], serde_json::json!(4));
        assert_eq!(metrics["ct_ga_asymmetry_post"], serde_json::json!(0.11));
        assert_eq!(metrics["used_fallback"], serde_json::json!(false));
    }

    #[test]
    fn filter_reads_standardized_metrics_prefer_governed_report() {
        let temp = tempfile::tempdir().expect("tempdir");
        let report_path = temp.path().join("filter_report.json");
        std::fs::write(
            &report_path,
            r#"{
                "schema_version": "bijux.fastq.filter_reads.report.v3",
                "stage": "fastq.filter_reads",
                "stage_id": "fastq.filter_reads",
                "tool_id": "fastp",
                "paired_mode": "single_end",
                "threads": 8,
                "input_r1": "reads.fastq.gz",
                "input_r2": null,
                "output_r1": "filtered.fastq.gz",
                "output_r2": null,
                "report_json": "filter_report.json",
                "max_n": 0,
                "max_n_fraction": 0.05,
                "max_n_count": 3,
                "low_complexity_threshold": 20.0,
                "entropy_threshold": 18.0,
                "n_policy": "drop",
                "polyx_policy": "trim",
                "contaminant_db": "contaminants.fa",
                "reads_in": 100,
                "reads_out": 95,
                "reads_dropped": 5,
                "reads_removed_by_n": 2,
                "reads_removed_by_entropy": 1,
                "reads_removed_low_complexity": 1,
                "reads_removed_by_kmer": 0,
                "reads_removed_contaminant_kmer": 0,
                "reads_removed_by_length": 1,
                "bases_in": 1000,
                "bases_out": 920,
                "pairs_in": null,
                "pairs_out": null,
                "mean_q_before": 28.0,
                "mean_q_after": 30.0,
                "runtime_s": 1.6,
                "memory_mb": 64.0,
                "exit_code": 0,
                "raw_backend_report": "fastp.filter.json",
                "raw_backend_report_format": "fastp_json",
                "backend_metrics": {
                    "passed_filter_reads": 95
                }
            }"#,
        )
        .expect("write report");

        let metrics = parse_filter_reads_metrics(temp.path());
        assert_eq!(metrics["tool"], serde_json::json!("fastp"));
        assert_eq!(metrics["threads"], serde_json::json!(8));
        assert_eq!(metrics["max_n_fraction"], serde_json::json!(0.05));
        assert_eq!(metrics["polyx_policy"], serde_json::json!("trim"));
        assert_eq!(metrics["reads_removed_by_n"], serde_json::json!(2));
        assert_eq!(
            metrics["raw_backend_report_format"],
            serde_json::json!("fastp_json")
        );
    }

    #[test]
    fn filter_low_complexity_standardized_metrics_prefer_governed_report() {
        let temp = tempfile::tempdir().expect("tempdir");
        let report_path = temp.path().join("low_complexity_report.json");
        std::fs::write(
            &report_path,
            serde_json::json!({
                "schema_version": "bijux.fastq.filter_low_complexity.report.v2",
                "stage": "fastq.filter_low_complexity",
                "stage_id": "fastq.filter_low_complexity",
                "tool_id": "bbduk",
                "paired_mode": "single_end",
                "threads": 8,
                "input_r1": "reads.fastq.gz",
                "input_r2": null,
                "output_r1": "filtered.fastq.gz",
                "output_r2": null,
                "report_json": "low_complexity_report.json",
                "entropy_threshold": 0.5,
                "polyx_threshold": 20,
                "reads_in": 100,
                "reads_out": 92,
                "reads_removed_low_complexity": 8,
                "bases_in": 1000,
                "bases_out": 910,
                "pairs_in": null,
                "pairs_out": null,
                "mean_q_before": 28.0,
                "mean_q_after": 29.0,
                "runtime_s": 1.1,
                "memory_mb": 64.0,
                "exit_code": 0,
                "raw_backend_report": "bbduk.low_complexity.stats",
                "raw_backend_report_format": "bbduk_stats",
                "backend_metrics": {
                    "reads_removed_reported": 8
                }
            })
            .to_string(),
        )
        .expect("write report");

        let metrics = parse_filter_low_complexity_metrics(temp.path());
        assert_eq!(metrics["tool"], serde_json::json!("bbduk"));
        assert_eq!(metrics["reads_removed_low_complexity"], serde_json::json!(8));
        assert_eq!(metrics["polyx_threshold"], serde_json::json!(20));
        assert_eq!(
            metrics["raw_backend_report_format"],
            serde_json::json!("bbduk_stats")
        );
    }

    #[test]
    fn extract_umis_standardized_metrics_prefer_governed_report() {
        let temp = tempfile::tempdir().expect("tempdir");
        let report_path = temp.path().join("umi_report.json");
        std::fs::write(
            &report_path,
            serde_json::json!({
                "schema_version": "bijux.fastq.extract_umis.report.v2",
                "stage": "fastq.extract_umis",
                "stage_id": "fastq.extract_umis",
                "tool_id": "umi_tools",
                "paired_mode": "paired_end",
                "threads": 2,
                "umi_pattern": "NNNNNNNN",
                "input_r1": "reads_R1.fastq.gz",
                "input_r2": "reads_R2.fastq.gz",
                "output_r1": "umi_reads_R1.fastq.gz",
                "output_r2": "umi_reads_R2.fastq.gz",
                "report_json": "umi_report.json",
                "reads_in": 200,
                "reads_out": 200,
                "bases_in": 20000,
                "bases_out": 20000,
                "pairs_in": 100,
                "pairs_out": 100,
                "reads_with_umi": 200,
                "mean_q_before": 30.0,
                "mean_q_after": 30.0,
                "runtime_s": 1.4,
                "memory_mb": 64.0,
                "exit_code": 0,
                "raw_backend_report": "umi_tools.extract.log",
                "raw_backend_report_format": "umi_tools_log",
                "backend_metrics": {
                    "reads_with_umi_fraction": 1.0
                }
            })
            .to_string(),
        )
        .expect("write report");

        let metrics = parse_extract_umis_metrics(temp.path());
        assert_eq!(metrics["tool"], serde_json::json!("umi_tools"));
        assert_eq!(metrics["umi_pattern"], serde_json::json!("NNNNNNNN"));
        assert_eq!(metrics["reads_with_umi"], serde_json::json!(200));
        assert_eq!(
            metrics["raw_backend_report_format"],
            serde_json::json!("umi_tools_log")
        );
    }

    #[test]
    fn profile_reads_standardized_metrics_prefer_governed_report() {
        let temp = tempfile::tempdir().expect("tempdir");
        let report_path = temp.path().join("qc.json");
        std::fs::write(
            &report_path,
            serde_json::json!({
                "schema_version": "bijux.fastq.profile_reads.report.v2",
                "stage": "fastq.profile_reads",
                "stage_id": "fastq.profile_reads",
                "tool_id": "seqkit_stats",
                "paired_mode": "paired_end",
                "threads": 6,
                "input_r1": "reads_R1.fastq.gz",
                "input_r2": "reads_R2.fastq.gz",
                "qc_json": "qc.json",
                "qc_tsv": "qc.tsv",
                "qc_plots_dir": "plots",
                "length_histogram_source": "seqkit_fx2tab",
                "reads_total": 200,
                "bases_total": 20000,
                "mean_q": 31.2,
                "gc_percent": 42.1,
                "length_histogram": [
                    {"length": 100, "count": 180},
                    {"length": 101, "count": 20}
                ],
                "mate_summaries": [
                    {"label": "reads_r1", "reads": 100, "bases": 10000, "mean_q": 31.0, "gc_percent": 41.9},
                    {"label": "reads_r2", "reads": 100, "bases": 10000, "mean_q": 31.4, "gc_percent": 42.3}
                ],
                "runtime_s": 1.5,
                "memory_mb": 20.0,
                "exit_code": 0,
                "raw_backend_report": "qc.tsv",
                "raw_backend_report_format": "seqkit_stats_tsv",
                "backend_metrics": [
                    {"schema_version": "bijux.seqkit.metrics.v1", "reads": 100, "bases": 10000, "mean_q": 31.0, "gc_percent": 41.9},
                    {"schema_version": "bijux.seqkit.metrics.v1", "reads": 100, "bases": 10000, "mean_q": 31.4, "gc_percent": 42.3}
                ]
            })
            .to_string(),
        )
        .expect("write report");

        let metrics = parse_profile_reads_metrics(temp.path());
        assert_eq!(metrics["tool"], serde_json::json!("seqkit_stats"));
        assert_eq!(metrics["paired_mode"], serde_json::json!("paired_end"));
        assert_eq!(metrics["threads"], serde_json::json!(6));
        assert_eq!(metrics["reads_total"], serde_json::json!(200));
        assert_eq!(metrics["length_histogram_bins"], serde_json::json!(2));
        assert_eq!(metrics["mate_summary_count"], serde_json::json!(2));
        assert_eq!(
            metrics["raw_backend_report_format"],
            serde_json::json!("seqkit_stats_tsv")
        );
    }

    #[test]
    fn profile_read_lengths_standardized_metrics_prefer_governed_report() {
        let temp = tempfile::tempdir().expect("tempdir");
        let report_path = temp.path().join("profile_read_lengths_report.json");
        std::fs::write(
            &report_path,
            serde_json::json!({
                "schema_version": "bijux.fastq.profile_read_lengths.report.v2",
                "stage": "fastq.profile_read_lengths",
                "stage_id": "fastq.profile_read_lengths",
                "tool_id": "seqkit_stats",
                "paired_mode": "paired_end",
                "threads": 2,
                "histogram_bins": 64,
                "input_r1": "reads_R1.fastq.gz",
                "input_r2": "reads_R2.fastq.gz",
                "length_distribution_tsv": "length_distribution.tsv",
                "length_distribution_json": "length_distribution.json",
                "report_json": "profile_read_lengths_report.json",
                "read_count": 200,
                "mean_read_length": 101.5,
                "max_read_length": 150,
                "distinct_lengths": 12,
                "histogram": [
                    {"read_length": 100, "count": 180},
                    {"read_length": 101, "count": 20}
                ],
                "runtime_s": 1.1,
                "memory_mb": 16.0,
                "exit_code": 0,
                "raw_backend_report": "length_distribution.tsv",
                "raw_backend_report_format": "seqkit_fx2tab_tsv"
            })
            .to_string(),
        )
        .expect("write report");

        let metrics = parse_profile_read_lengths_metrics(temp.path());
        assert_eq!(metrics["tool"], serde_json::json!("seqkit_stats"));
        assert_eq!(metrics["paired_mode"], serde_json::json!("paired_end"));
        assert_eq!(metrics["histogram_bins"], serde_json::json!(64));
        assert_eq!(metrics["read_count"], serde_json::json!(200));
        assert_eq!(metrics["histogram_entry_count"], serde_json::json!(2));
        assert_eq!(
            metrics["raw_backend_report_format"],
            serde_json::json!("seqkit_fx2tab_tsv")
        );
    }

    #[test]
    fn profile_overrepresented_standardized_metrics_prefer_governed_report() {
        let temp = tempfile::tempdir().expect("tempdir");
        let report_path = temp.path().join("overrepresented_report.json");
        std::fs::write(
            &report_path,
            serde_json::json!({
                "schema_version": "bijux.fastq.profile_overrepresented.report.v2",
                "stage": "fastq.profile_overrepresented_sequences",
                "stage_id": "fastq.profile_overrepresented_sequences",
                "tool_id": "fastqc",
                "paired_mode": "paired_end",
                "threads": 4,
                "top_k": 25,
                "input_r1": "reads_R1.fastq.gz",
                "input_r2": "reads_R2.fastq.gz",
                "overrepresented_sequences_tsv": "overrepresented_sequences.tsv",
                "overrepresented_sequences_json": "overrepresented_sequences.json",
                "report_json": "overrepresented_report.json",
                "sequence_count": 25,
                "flagged_sequences": 3,
                "top_fraction": 0.12,
                "rows": [
                    {"sequence": "ACGT", "count": 12, "fraction": 0.12, "flag": "overrepresented"}
                ],
                "runtime_s": 1.4,
                "memory_mb": 48.0,
                "exit_code": 0,
                "raw_backend_report": null,
                "raw_backend_report_format": null
            })
            .to_string(),
        )
        .expect("write report");

        let metrics = parse_profile_overrepresented_metrics(temp.path());
        assert_eq!(metrics["tool"], serde_json::json!("fastqc"));
        assert_eq!(metrics["paired_mode"], serde_json::json!("paired_end"));
        assert_eq!(metrics["top_k"], serde_json::json!(25));
        assert_eq!(metrics["sequence_count"], serde_json::json!(25));
        assert_eq!(metrics["flagged_sequences"], serde_json::json!(3));
        assert_eq!(metrics["row_count"], serde_json::json!(1));
    }

    #[test]
    fn trim_reads_standardized_metrics_prefer_governed_report() {
        let temp = tempfile::tempdir().expect("tempdir");
        let report_path = temp.path().join("trim_report.json");
        std::fs::write(
            &report_path,
            serde_json::json!({
                "schema_version": "bijux.fastq.trim_reads.report.v2",
                "stage": "fastq.trim_reads",
                "stage_id": "fastq.trim_reads",
                "tool_id": "fastp",
                "paired_mode": "single_end",
                "threads": 4,
                "input_r1": "reads.fastq.gz",
                "input_r2": null,
                "output_r1": "trimmed.fastq.gz",
                "output_r2": null,
                "min_length": 30,
                "quality_cutoff": 20,
                "adapter_policy": "bank",
                "polyx_policy": "trim",
                "n_policy": "drop",
                "contaminant_policy": "none",
                "adapter_bank_id": "illumina",
                "adapter_bank_hash": "sha256:adapter",
                "adapter_preset": "default",
                "adapter_overrides": {
                    "enable": ["AGATCGGAAGAGC"],
                    "disable": ["polyA"]
                },
                "polyx_bank_id": "polyx",
                "polyx_bank_hash": "sha256:polyx",
                "polyx_preset": "illumina_twocolor",
                "contaminant_bank_id": null,
                "contaminant_bank_hash": null,
                "contaminant_preset": null,
                "reads_in": 100,
                "reads_out": 92,
                "bases_in": 1000,
                "bases_out": 850,
                "pairs_in": null,
                "pairs_out": null,
                "mean_q_before": 28.0,
                "mean_q_after": 30.0,
                "runtime_s": 5.0,
                "memory_mb": 64.0,
                "raw_backend_report": "trim.fastp.json",
                "raw_backend_report_format": "fastp_json"
            })
            .to_string(),
        )
        .expect("write report");

        let metrics = parse_trim_reads_metrics(temp.path());
        assert_eq!(metrics["tool"], serde_json::json!("fastp"));
        assert_eq!(metrics["threads"], serde_json::json!(4));
        assert_eq!(metrics["adapter_policy"], serde_json::json!("bank"));
        assert_eq!(
            metrics["adapter_overrides"],
            serde_json::json!({
                "enable": ["AGATCGGAAGAGC"],
                "disable": ["polyA"]
            })
        );
        assert_eq!(metrics["reads_in"], serde_json::json!(100));
        assert_eq!(metrics["reads_out"], serde_json::json!(92));
        assert_eq!(
            metrics["raw_backend_report_format"],
            serde_json::json!("fastp_json")
        );
    }

    #[test]
    fn merge_pairs_standardized_metrics_prefer_governed_report() {
        let temp = tempfile::tempdir().expect("tempdir");
        let report_path = temp.path().join("merge_report.json");
        std::fs::write(
            &report_path,
            serde_json::json!({
                "schema_version": "bijux.fastq.merge_pairs.report.v2",
                "stage": "fastq.merge_pairs",
                "stage_id": "fastq.merge_pairs",
                "tool_id": "pear",
                "paired_mode": "paired_end",
                "merge_engine": "pear",
                "threads": 4,
                "merge_overlap": 22,
                "min_len": 120,
                "unmerged_read_policy": "omit_unmerged_pairs",
                "input_r1": "reads_R1.fastq.gz",
                "input_r2": "reads_R2.fastq.gz",
                "merged_reads": "pear.assembled.fastq",
                "unmerged_reads_r1": null,
                "unmerged_reads_r2": null,
                "reads_r1": 100,
                "reads_r2": 100,
                "reads_merged": 88,
                "reads_unmerged": 12,
                "merge_rate": 0.88,
                "runtime_s": 2.3,
                "memory_mb": 48.0,
                "raw_backend_report": null,
                "raw_backend_report_format": null
            })
            .to_string(),
        )
        .expect("write report");

        let metrics = parse_merge_pairs_metrics(temp.path());
        assert_eq!(metrics["tool"], serde_json::json!("pear"));
        assert_eq!(metrics["paired_mode"], serde_json::json!("paired_end"));
        assert_eq!(metrics["merge_engine"], serde_json::json!("pear"));
        assert_eq!(metrics["threads"], serde_json::json!(4));
        assert_eq!(metrics["merge_overlap"], serde_json::json!(22));
        assert_eq!(metrics["min_length"], serde_json::json!(120));
        assert_eq!(
            metrics["unmerged_read_policy"],
            serde_json::json!("omit_unmerged_pairs")
        );
        assert_eq!(metrics["reads_r1"], serde_json::json!(100));
        assert_eq!(metrics["reads_r2"], serde_json::json!(100));
        assert_eq!(metrics["reads_merged"], serde_json::json!(88));
        assert_eq!(metrics["reads_unmerged"], serde_json::json!(12));
        assert_eq!(metrics["merge_rate"], serde_json::json!(0.88));
        assert_eq!(metrics["raw_backend_report_format"], serde_json::Value::Null);
    }

    #[test]
    fn trim_polyg_standardized_metrics_prefer_governed_report() {
        let temp = tempfile::tempdir().expect("tempdir");
        let report_path = temp.path().join("trim_polyg_tails_report.json");
        std::fs::write(
            &report_path,
            serde_json::json!({
                "schema_version": "bijux.fastq.trim_polyg_tails.report.v2",
                "stage": "fastq.trim_polyg_tails",
                "stage_id": "fastq.trim_polyg_tails",
                "tool_id": "fastp",
                "paired_mode": "single_end",
                "threads": 6,
                "trim_polyg": true,
                "min_polyg_run": 10,
                "input_r1": "reads.fastq.gz",
                "input_r2": null,
                "output_r1": "trimmed.fastq.gz",
                "output_r2": null,
                "reads_in": 100,
                "reads_out": 96,
                "bases_in": 1000,
                "bases_out": 910,
                "pairs_in": null,
                "pairs_out": null,
                "mean_q_before": 28.0,
                "mean_q_after": 29.4,
                "bases_trimmed_polyg": 90,
                "polyx_bank_id": "polyx",
                "polyx_bank_hash": "sha256:polyx",
                "polyx_preset": "illumina_twocolor",
                "runtime_s": 4.5,
                "memory_mb": 72.0,
                "raw_backend_report": "trim.fastp.json",
                "raw_backend_report_format": "fastp_json",
                "backend_metrics": {
                    "schema_version": "bijux.fastp.metrics.v1",
                    "passed_filter_reads": 96
                }
            })
            .to_string(),
        )
        .expect("write report");

        let metrics = parse_trim_polyg_metrics(temp.path());
        assert_eq!(metrics["tool"], serde_json::json!("fastp"));
        assert_eq!(metrics["threads"], serde_json::json!(6));
        assert_eq!(metrics["trim_polyg"], serde_json::json!(true));
        assert_eq!(metrics["reads_in"], serde_json::json!(100));
        assert_eq!(metrics["bases_trimmed_polyg"], serde_json::json!(90));
        assert_eq!(
            metrics["polyx_bank_hash"],
            serde_json::json!("sha256:polyx")
        );
    }

    #[test]
    fn normalize_primers_standardized_metrics_prefer_governed_report() {
        let temp = tempfile::tempdir().expect("tempdir");
        let report_path = temp.path().join("normalize_primers_report.json");
        std::fs::write(
            &report_path,
            serde_json::json!({
                "schema_version": "bijux.fastq.normalize_primers.report.v2",
                "stage": "fastq.normalize_primers",
                "stage_id": "fastq.normalize_primers",
                "tool_id": "cutadapt",
                "paired_mode": "paired_end",
                "primer_set_id": "16S_universal_v1",
                "marker_id": "16S",
                "primer_fasta": "assets/reference/primers/16S_universal_v1.fasta",
                "orientation_policy": "normalize_to_forward_primer",
                "max_mismatch_rate": 0.1,
                "min_overlap_bp": 10,
                "input_r1": "reads_R1.fastq.gz",
                "input_r2": "reads_R2.fastq.gz",
                "output_r1": "normalized_R1.fastq.gz",
                "output_r2": "normalized_R2.fastq.gz",
                "reads_in": 200,
                "reads_out": 200,
                "bases_in": 1000,
                "bases_out": 980,
                "pairs_in": 100,
                "pairs_out": 100,
                "primer_trimmed_reads": 190,
                "primer_trimmed_fraction": 0.95,
                "orientation_forward_fraction": 0.93,
                "primer_orientation_report": "primer_orientation.tsv",
                "primer_stats_json": "primer_stats.json",
                "raw_backend_report": "primer_stats.json",
                "raw_backend_report_format": "cutadapt_json",
                "runtime_s": 2.4,
                "memory_mb": 80.0,
                "used_fallback": false,
                "backend_metrics": {}
            })
            .to_string(),
        )
        .expect("write report");

        let metrics = parse_normalize_primers_metrics(temp.path());
        assert_eq!(metrics["tool"], serde_json::json!("cutadapt"));
        assert_eq!(
            metrics["primer_set_id"],
            serde_json::json!("16S_universal_v1")
        );
        assert_eq!(metrics["primer_trimmed_reads"], serde_json::json!(190));
        assert_eq!(
            metrics["orientation_forward_fraction"],
            serde_json::json!(0.93)
        );
    }

    #[test]
    fn normalize_abundance_standardized_metrics_prefer_governed_report() {
        let temp = tempfile::tempdir().expect("tempdir");
        let report_path = temp.path().join("normalize_abundance_report.json");
        std::fs::write(
            &report_path,
            serde_json::json!({
                "schema_version": "bijux.fastq.normalize_abundance.report.v2",
                "stage": "fastq.normalize_abundance",
                "stage_id": "fastq.normalize_abundance",
                "tool_id": "seqkit",
                "method": "counts_per_million",
                "input_table": "otu_abundance.tsv",
                "normalized_abundance_tsv": "abundance_normalized.tsv",
                "expected_columns": ["sample_id", "feature_id", "abundance"],
                "input_value_column": "abundance",
                "normalized_value_column": "counts_per_million",
                "compositional_rule": "per_sample_sum_to_one_million",
                "scale_factor": 1_000_000.0,
                "table_rows": 12,
                "sample_count": 3,
                "feature_count": 4,
                "zero_fraction": 0.25,
                "per_sample_sums": [["sample_a", 1_000_000.0], ["sample_b", 1_000_000.0]],
                "runtime_s": 1.8,
                "memory_mb": 24.0,
                "raw_backend_report": null,
                "raw_backend_report_format": null,
                "used_fallback": false,
                "backend_metrics": {"metric_set": {"table_rows": 12}}
            })
            .to_string(),
        )
        .expect("write report");

        let metrics = parse_normalize_abundance_metrics(temp.path());
        assert_eq!(metrics["tool"], serde_json::json!("seqkit"));
        assert_eq!(metrics["method"], serde_json::json!("counts_per_million"));
        assert_eq!(metrics["table_rows"], serde_json::json!(12));
        assert_eq!(metrics["sample_count"], serde_json::json!(3));
        assert_eq!(metrics["per_sample_sum_count"], serde_json::json!(2));
    }

    #[test]
    fn screen_taxonomy_standardized_metrics_prefer_governed_report() {
        let temp = tempfile::tempdir().expect("tempdir");
        let report_path = temp.path().join("kraken2.classifications.json");
        std::fs::write(
            &report_path,
            serde_json::json!({
                "schema_version": "bijux.fastq.screen_taxonomy.report.v2",
                "stage": "fastq.screen_taxonomy",
                "stage_id": "fastq.screen_taxonomy",
                "tool_id": "kraken2",
                "paired_mode": "single_end",
                "threads": 8,
                "classifier": "kraken2",
                "report_format": "kraken_report",
                "assignment_format": "kraken_assignments",
                "database_catalog_id": "taxonomy_reference",
                "database_artifact_id": "taxonomy_db",
                "database_build_id": "build-2026-03",
                "database_digest": "sha256:taxonomy-db",
                "database_namespace": "read_screening",
                "database_scope": "read_screening",
                "minimum_confidence": 0.05,
                "emit_unclassified": true,
                "input_r1": "reads.fastq.gz",
                "input_r2": null,
                "screen_report_tsv": "kraken2.report.tsv",
                "classification_report_json": "kraken2.classifications.json",
                "reads_in": 100,
                "reads_out": 100,
                "bases_in": 1000,
                "bases_out": 1000,
                "pairs_in": 0,
                "pairs_out": 0,
                "contamination_rate": 0.23,
                "classified_fraction": 0.77,
                "unclassified_fraction": 0.23,
                "summary_entries": [
                    {"label": "unclassified", "percent": 23.0},
                    {"label": "bacteria", "percent": 77.0}
                ],
                "top_taxa": [
                    {"label": "bacteria", "percent": 77.0}
                ],
                "runtime_s": 8.0,
                "memory_mb": 256.0
            })
            .to_string(),
        )
        .expect("write report");

        let metrics = parse_screen_taxonomy_metrics(temp.path());
        assert_eq!(metrics["tool"], serde_json::json!("kraken2"));
        assert_eq!(metrics["classifier"], serde_json::json!("kraken2"));
        assert_eq!(metrics["contamination_rate"], serde_json::json!(0.23));
        assert_eq!(
            metrics["top_taxa"][0]["label"],
            serde_json::json!("bacteria")
        );
        assert_eq!(
            metrics["database_digest"],
            serde_json::json!("sha256:taxonomy-db")
        );
    }

    #[test]
    fn deplete_rrna_standardized_metrics_prefer_governed_report() {
        let temp = tempfile::tempdir().expect("tempdir");
        let report_path = temp.path().join("rrna_report.json");
        std::fs::write(
            &report_path,
            serde_json::json!({
                "schema_version": "bijux.fastq.deplete_rrna.report.v2",
                "stage": "fastq.deplete_rrna",
                "stage_id": "fastq.deplete_rrna",
                "tool_id": "sortmerna",
                "paired_mode": "single_end",
                "threads": 4,
                "rrna_db": "/refs/silva",
                "database_artifact_id": "silva_nr99",
                "database_build_id": "2026.03",
                "screening_engine": "sortmerna",
                "report_format": "summary_tsv_and_json",
                "emit_removed_reads": false,
                "min_identity": 0.95,
                "input_r1": "reads.fastq.gz",
                "input_r2": null,
                "output_r1": "rrna_filtered.fastq.gz",
                "output_r2": null,
                "rrna_report_tsv": "rrna_report.tsv",
                "rrna_report_json": "rrna_report.json",
                "reads_in": 100,
                "reads_out": 64,
                "reads_removed": 36,
                "bases_in": 1000,
                "bases_out": 620,
                "bases_removed": 380,
                "pairs_in": null,
                "pairs_out": null,
                "rrna_fraction_removed": 0.36,
                "runtime_s": 5.0,
                "memory_mb": 64.0,
                "exit_code": 0,
                "raw_backend_report": null,
                "raw_backend_report_format": null,
                "backend_metrics": {"reads_removed": 36}
            })
            .to_string(),
        )
        .expect("write report");

        let metrics = parse_deplete_rrna_metrics(temp.path());
        assert_eq!(metrics["tool"], serde_json::json!("sortmerna"));
        assert_eq!(metrics["database_artifact_id"], serde_json::json!("silva_nr99"));
        assert_eq!(metrics["reads_removed"], serde_json::json!(36));
        assert_eq!(metrics["rrna_fraction_removed"], serde_json::json!(0.36));
    }

    #[test]
    fn deplete_reference_contaminants_standardized_metrics_prefer_governed_report() {
        let temp = tempfile::tempdir().expect("tempdir");
        let report_path = temp.path().join("contaminant_screen_report.json");
        std::fs::write(
            &report_path,
            serde_json::json!({
                "schema_version": "bijux.fastq.deplete_reference_contaminants.report.v2",
                "stage": "fastq.deplete_reference_contaminants",
                "stage_id": "fastq.deplete_reference_contaminants",
                "tool_id": "bowtie2",
                "paired_mode": "single_end",
                "threads": 4,
                "reference_catalog_id": "contaminant_reference",
                "contaminant_reference": "phix_and_spikeins",
                "index_artifact": "reference_index",
                "reference_index_backend": "bowtie2_build",
                "reference_build_id": "2026.03",
                "reference_digest": "sha256:contaminants",
                "retain_unmapped_pairs": false,
                "input_r1": "reads.fastq.gz",
                "input_r2": null,
                "output_r1": "contaminant_screened.fastq.gz",
                "output_r2": null,
                "report_json": "contaminant_screen_report.json",
                "reads_in": 100,
                "reads_out": 72,
                "reads_removed": 28,
                "bases_in": 1000,
                "bases_out": 700,
                "bases_removed": 300,
                "pairs_in": null,
                "pairs_out": null,
                "contaminant_fraction_removed": 0.28,
                "runtime_s": 5.0,
                "memory_mb": 64.0,
                "exit_code": 0,
                "raw_backend_report": "bowtie2.contaminant.metrics.txt",
                "raw_backend_report_format": "bowtie2_met_file",
                "backend_metrics": {"reads_removed": 28}
            })
            .to_string(),
        )
        .expect("write report");

        let metrics = parse_deplete_reference_contaminants_metrics(temp.path());
        assert_eq!(metrics["tool"], serde_json::json!("bowtie2"));
        assert_eq!(
            metrics["contaminant_reference"],
            serde_json::json!("phix_and_spikeins")
        );
        assert_eq!(metrics["reads_removed"], serde_json::json!(28));
        assert_eq!(
            metrics["contaminant_fraction_removed"],
            serde_json::json!(0.28)
        );
    }

    #[test]
    fn deplete_host_standardized_metrics_prefer_governed_report() {
        let temp = tempfile::tempdir().expect("tempdir");
        let report_path = temp.path().join("host_depletion_report.json");
        std::fs::write(
            &report_path,
            r#"{
                "schema_version": "bijux.fastq.deplete_host.report.v2",
                "stage": "fastq.deplete_host",
                "stage_id": "fastq.deplete_host",
                "tool_id": "bowtie2",
                "paired_mode": "single_end",
                "threads": 4,
                "reference_scope": "host",
                "reference_catalog_id": "host_reference",
                "reference_index_artifact_id": "reference_index",
                "reference_index_backend": "bowtie2_build",
                "reference_build_id": "2026.03",
                "reference_digest": "sha256:host",
                "masking_policy": "unmasked",
                "decoy_policy": "none",
                "decoy_catalog_id": null,
                "identity_threshold": 0.95,
                "retained_read_policy": "keep_non_host_reads",
                "emit_removed_reads": true,
                "report_format": "bowtie2_metrics_file",
                "retain_unmapped_pairs": false,
                "input_r1": "reads.fastq.gz",
                "input_r2": null,
                "output_r1": "host_depleted.fastq.gz",
                "output_r2": null,
                "removed_host_r1": "removed_host.fastq.gz",
                "removed_host_r2": null,
                "report_json": "host_depletion_report.json",
                "reads_in": 100,
                "reads_out": 70,
                "reads_removed": 30,
                "bases_in": 1000,
                "bases_out": 680,
                "bases_removed": 320,
                "pairs_in": null,
                "pairs_out": null,
                "host_fraction_removed": 0.30,
                "runtime_s": 5.0,
                "memory_mb": 64.0,
                "exit_code": 0,
                "raw_backend_report": "bowtie2.host.metrics.txt",
                "raw_backend_report_format": "bowtie2_met_file",
                "backend_metrics": {"reads_removed": 30}
            }"#,
        )
        .expect("write report");

        let metrics = parse_deplete_host_metrics(temp.path());
        assert_eq!(metrics["tool"], serde_json::json!("bowtie2"));
        assert_eq!(metrics["reference_catalog_id"], serde_json::json!("host_reference"));
        assert_eq!(metrics["reads_removed"], serde_json::json!(30));
        assert_eq!(metrics["host_fraction_removed"], serde_json::json!(0.30));
    }

    #[test]
    fn report_qc_standardized_metrics_prefer_governed_report() {
        let temp = tempfile::tempdir().expect("tempdir");
        let report_path = temp.path().join("report_qc_report.json");
        std::fs::write(
            &report_path,
            serde_json::json!({
                "schema_version": "bijux.fastq.report_qc.report.v2",
                "stage": "fastq.report_qc",
                "stage_id": "fastq.report_qc",
                "tool_id": "multiqc",
                "paired_mode": "single_end",
                "aggregation_engine": "multiqc",
                "aggregation_scope": "governed_qc_artifacts",
                "reads_in": 100,
                "reads_out": 100,
                "bases_in": 1000,
                "bases_out": 1000,
                "pairs_in": 0,
                "pairs_out": 0,
                "mean_q": 31.0,
                "contamination_rate": 0.0,
                "multiqc_sample_count": 2,
                "multiqc_module_count": 5,
                "multiqc_report": "multiqc_report.html",
                "multiqc_data": "multiqc_data",
                "governed_qc_input_count": 3,
                "governed_qc_contributor_stage_ids": ["fastq.trim_reads", "fastq.validate_reads"],
                "governed_qc_contributor_tool_ids": ["fastp", "fastqvalidator"],
                "governed_qc_contributors": [],
                "governed_qc_lineage_hash": "lineage",
                "governed_qc_inputs_manifest": "governed_qc_inputs_manifest.json",
                "runtime_s": 3.0,
                "memory_mb": 128.0,
                "exit_code": 0
            })
            .to_string(),
        )
        .expect("write report");

        let metrics = parse_report_qc_metrics(temp.path());
        assert_eq!(metrics["tool"], serde_json::json!("multiqc"));
        assert_eq!(metrics["aggregation_engine"], serde_json::json!("multiqc"));
        assert_eq!(
            metrics["aggregation_scope"],
            serde_json::json!("governed_qc_artifacts")
        );
        assert_eq!(metrics["reads_in"], serde_json::json!(100));
        assert_eq!(metrics["mean_q"], serde_json::json!(31.0));
        assert_eq!(metrics["governed_qc_input_count"], serde_json::json!(3));
        assert_eq!(metrics["multiqc_module_count"], serde_json::json!(5));
        assert_eq!(
            metrics["governed_qc_contributor_tool_ids"],
            serde_json::json!(["fastp", "fastqvalidator"])
        );
        assert_eq!(metrics["report_json"], serde_json::json!(report_path));
    }

    #[test]
    fn remove_duplicates_standardized_metrics_prefer_governed_report() {
        let temp = tempfile::tempdir().expect("tempdir");
        let report_path = temp.path().join("deduplicate_report.json");
        std::fs::write(
            &report_path,
            serde_json::json!({
                "schema_version": "bijux.fastq.remove_duplicates.report.v2",
                "stage": "fastq.remove_duplicates",
                "stage_id": "fastq.remove_duplicates",
                "tool_id": "clumpify",
                "paired_mode": "single_end",
                "threads": 4,
                "dedup_mode": "optical_aware",
                "keep_order": false,
                "input_r1": "reads.fastq.gz",
                "input_r2": null,
                "output_r1": "dedup.fastq.gz",
                "output_r2": null,
                "reads_in": 100,
                "reads_out": 84,
                "reads_in_r2": null,
                "reads_out_r2": null,
                "pairs_in": null,
                "pairs_out": null,
                "pair_count_match": null,
                "duplicates_removed": 16,
                "dedup_rate": 0.16,
                "duplicate_classes_tsv": "duplicate_classes.tsv",
                "duplicate_provenance_json": "duplicate_provenance.json",
                "duplicate_classes": [
                    {"class": "duplicate", "reads_removed": 16, "paired_mode": "single_end"}
                ],
                "raw_backend_report": "clumpify.log",
                "raw_backend_report_format": "clumpify_log",
                "runtime_s": null,
                "memory_mb": null
            })
            .to_string(),
        )
        .expect("write report");

        let metrics = parse_remove_duplicates_metrics(temp.path());
        assert_eq!(metrics["tool"], serde_json::json!("clumpify"));
        assert_eq!(metrics["threads"], serde_json::json!(4));
        assert_eq!(metrics["dedup_mode"], serde_json::json!("optical_aware"));
        assert_eq!(metrics["duplicates_removed"], serde_json::json!(16));
        assert_eq!(metrics["duplicate_class_count"], serde_json::json!(1));
        assert_eq!(metrics["raw_backend_report"], serde_json::json!("clumpify.log"));
    }

    #[test]
    fn remove_duplicates_uses_governed_report_metrics_policy() {
        assert_eq!(
            required_metrics_keys("fastq.remove_duplicates"),
            &[
                "schema_version",
                "stage",
                "tool",
                "threads",
                "dedup_mode",
                "keep_order",
                "reads_in",
                "reads_out",
                "duplicates_removed",
                "dedup_rate",
                "duplicate_class_count",
            ]
        );
    }
}

pub(super) fn required_fastq_tools() -> Result<std::collections::BTreeSet<String>> {
    let raw = std::fs::read_to_string(
        crate::support::repo_root::resolve_repo_root()?.join("configs/ci/tools/required_tools.toml"),
    )?;
    let parsed: toml::Value = toml::from_str(&raw)?;
    let mut set = std::collections::BTreeSet::new();
    let items = parsed
        .get("required_tools")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| anyhow!("missing required_tools in required_tools.toml"))?;
    for item in items {
        if let Some(id) = item.as_str() {
            set.insert(id.to_string());
        }
    }
    Ok(set)
}

pub(super) fn enforce_screen_db_governance(planned: &ExecutionStep) -> Result<()> {
    let stage = planned.step_id.as_str();
    if !matches!(
        stage,
        "fastq.screen_taxonomy"
            | "fastq.deplete_rrna"
            | "fastq.deplete_host"
            | "fastq.deplete_reference_contaminants"
    ) {
        return Ok(());
    }
    let template = planned.command.template.join(" ");
    if template.contains("http://") || template.contains("https://") {
        return Err(anyhow!(
            "{stage} may not fetch databases over network at runtime; use pre-mounted references"
        ));
    }
    if template.contains("download") || template.contains("pull") {
        return Err(anyhow!(
            "{stage} command contains database fetch verbs; require immutable pre-resolved DB paths"
        ));
    }
    Ok(())
}

pub(super) fn required_metrics_keys(stage_id: &str) -> &'static [&'static str] {
    match stage_id {
        "fastq.validate_reads" => &[
            "schema_version",
            "stage",
            "validator",
            "failure_class",
            "strict_pass",
            "exit_code",
        ],
        "fastq.index_reference" => &[
            "schema_version",
            "stage",
            "tool",
            "reference_bytes",
            "index_bytes",
            "index_file_count",
        ],
        "fastq.detect_adapters" => &[
            "schema_version",
            "stage",
            "tool",
            "candidate_adapter_count",
            "adapter_inference",
        ],
        "fastq.trim_reads" => &[
            "schema_version",
            "stage",
            "tool",
            "threads",
            "adapter_policy",
            "adapter_overrides",
            "reads_in",
            "reads_out",
        ],
        "fastq.trim_terminal_damage" => &[
            "schema_version",
            "stage",
            "tool",
            "threads",
            "execution_policy",
            "trim_5p_bases",
            "trim_3p_bases",
            "udg_classification",
            "ct_ga_asymmetry_pre",
            "ct_ga_asymmetry_post",
        ],
        "fastq.merge_pairs" => &[
            "schema_version",
            "stage",
            "tool",
            "paired_mode",
            "merge_engine",
            "threads",
            "merge_overlap",
            "min_length",
            "unmerged_read_policy",
            "reads_r1",
            "reads_r2",
            "reads_merged",
            "reads_unmerged",
            "merge_rate",
            "raw_backend_report_format",
        ],
        "fastq.remove_duplicates" => &[
            "schema_version",
            "stage",
            "tool",
            "threads",
            "dedup_mode",
            "keep_order",
            "reads_in",
            "reads_out",
            "duplicates_removed",
            "dedup_rate",
            "duplicate_class_count",
        ],
        "fastq.correct_errors" => &[
            "schema_version",
            "stage",
            "tool",
            "correction_engine",
            "corrected_reads",
            "kmer_fix_rate",
        ],
        "fastq.filter_reads" => &[
            "schema_version",
            "stage",
            "tool",
            "reads_in",
            "reads_out",
            "reads_dropped",
        ],
        "fastq.filter_low_complexity" => &[
            "schema_version",
            "stage",
            "tool",
            "reads_in",
            "reads_out",
            "reads_removed_low_complexity",
        ],
        "fastq.extract_umis" => &[
            "schema_version",
            "stage",
            "tool",
            "reads_in",
            "reads_out",
            "reads_with_umi",
        ],
        "fastq.profile_reads" => &[
            "schema_version",
            "stage",
            "tool",
            "reads_total",
            "bases_total",
            "length_histogram_bins",
        ],
        "fastq.profile_read_lengths" => &[
            "schema_version",
            "stage",
            "tool",
            "read_count",
            "mean_read_length",
            "histogram_entry_count",
        ],
        "fastq.profile_overrepresented_sequences" => &[
            "schema_version",
            "stage",
            "tool",
            "sequence_count",
            "flagged_sequences",
            "top_fraction",
        ],
        "fastq.trim_polyg_tails" => &[
            "schema_version",
            "stage",
            "tool",
            "trim_polyg",
            "reads_in",
            "reads_out",
        ],
        "fastq.screen_taxonomy" => &[
            "schema_version",
            "stage",
            "tool",
            "classifier",
            "contamination_rate",
            "top_taxa",
        ],
        "fastq.deplete_rrna" => &[
            "schema_version",
            "stage",
            "tool",
            "database_artifact_id",
            "reads_removed",
            "rrna_fraction_removed",
        ],
        "fastq.deplete_reference_contaminants" => &[
            "schema_version",
            "stage",
            "tool",
            "contaminant_reference",
            "reads_removed",
            "contaminant_fraction_removed",
        ],
        "fastq.deplete_host" => &[
            "schema_version",
            "stage",
            "tool",
            "host_fraction_removed",
            "reads_removed",
        ],
        "fastq.report_qc" => &[
            "schema_version",
            "stage",
            "tool",
            "aggregation_engine",
            "aggregation_scope",
            "governed_qc_input_count",
            "multiqc_report",
            "multiqc_data",
            "report_json",
        ],
        "fastq.normalize_abundance" => &[
            "schema_version",
            "stage",
            "tool",
            "method",
            "table_rows",
            "sample_count",
            "zero_fraction",
        ],
        _ => &["schema_version", "stage"],
    }
}

pub(super) fn enforce_metrics_schema(stage_root: &std::path::Path, stage_id: &str) -> Result<()> {
    let metrics_path = stage_root.join("metrics.json");
    let raw = std::fs::read_to_string(&metrics_path)
        .with_context(|| format!("reading metrics {}", metrics_path.display()))?;
    let parsed: serde_json::Value = serde_json::from_str(&raw)
        .with_context(|| format!("parsing metrics {}", metrics_path.display()))?;
    let required = required_metrics_keys(stage_id);
    for key in required {
        if parsed.get(*key).is_none() {
            return Err(anyhow!(
                "metrics schema violation for {stage_id}: missing key `{key}` in {}",
                metrics_path.display()
            ));
        }
    }
    Ok(())
}

fn count_fastq_reads_if_plain(path: &std::path::Path) -> Option<u64> {
    let ext = path
        .extension()
        .and_then(|x| x.to_str())
        .unwrap_or_default();
    if ext == "gz" {
        return None;
    }
    let file = std::fs::File::open(path).ok()?;
    let lines = std::io::BufReader::new(file).lines().count() as u64;
    Some(lines / 4)
}

pub(super) fn write_retention_report(
    stage_root: &std::path::Path,
    planned: &ExecutionStep,
) -> Result<()> {
    let out_dir = stage_root.join("out");
    let mut rows = vec!["artifact\treads_estimate".to_string()];
    if let Ok(entries) = std::fs::read_dir(&out_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .filter(|value| !value.trim().is_empty())
                .map(ToOwned::to_owned)
                .unwrap_or_else(|| path.display().to_string());
            let reads = count_fastq_reads_if_plain(&path)
                .map_or_else(|| "na".to_string(), |x| x.to_string());
            rows.push(format!("{name}\t{reads}"));
        }
    }
    let payload = rows.join("\n") + "\n";
    std::fs::write(stage_root.join("retention_report.tsv"), payload)?;
    std::fs::write(
        stage_root.join("retention_report.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "schema_version": "bijux.fastq.retention_report.v1",
            "stage_id": planned.step_id.0,
            "out_dir": out_dir,
            "artifacts": rows.len().saturating_sub(1),
        }))?,
    )?;
    Ok(())
}

pub(super) fn classify_failure_hint(stage_id: &str, stdout: &str, stderr: &str) -> String {
    let merged = format!("{stdout}\n{stderr}").to_ascii_lowercase();
    if merged.contains("out of memory") || merged.contains("killed") {
        return "resource_exhausted_memory".to_string();
    }
    if merged.contains("no space left") {
        return "resource_exhausted_disk".to_string();
    }
    if merged.contains("permission denied") {
        return "filesystem_permissions".to_string();
    }
    if merged.contains("not found") || merged.contains("no such file") {
        return "missing_input_or_tool".to_string();
    }
    format!("{stage_id}_execution_failure")
}

pub(super) fn write_retry_policy(root: &std::path::Path) -> Result<()> {
    let payload = serde_json::json!({
        "schema_version": "bijux.retry_policy.v1",
        "max_retries": 0,
        "note": "fastq preprocessing stages are deterministic and should not auto-retry by default"
    });
    std::fs::write(
        root.join("retry_policy.json"),
        serde_json::to_vec_pretty(&payload)?,
    )?;
    Ok(())
}
use super::*;
