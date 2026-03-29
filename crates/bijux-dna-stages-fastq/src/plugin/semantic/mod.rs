use super::*;

pub(super) fn observed_semantic_metrics(
    plan: &StagePlanV1,
    artifacts: &[ArtifactRef],
) -> serde_json::Value {
    if plan.stage_id.as_str() == "fastq.merge_pairs" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_merge_pairs_report(&raw_report) {
                    return serde_json::json!({
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
                        "merged_reads": report.merged_reads,
                        "unmerged_reads_r1": report.unmerged_reads_r1,
                        "unmerged_reads_r2": report.unmerged_reads_r2,
                        "raw_backend_report": report.raw_backend_report,
                        "raw_backend_report_format": report.raw_backend_report_format,
                    });
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.report_qc" {
        let multiqc_metrics = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "multiqc_data")
            .map(|artifact| artifact.path.join("multiqc_general_stats.json"))
            .filter(|path| path.exists())
            .and_then(|path| fs::read_to_string(path).ok())
            .and_then(|raw| parse_multiqc_general_stats_metrics(&raw).ok());
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_report_qc_report(&raw_report) {
                    return serde_json::json!({
                        "aggregation_engine": report.aggregation_engine,
                        "aggregation_scope": report.aggregation_scope,
                        "paired_mode": report.paired_mode,
                        "lineage_hash": report.governed_qc_lineage_hash,
                        "contributor_artifact_count": report.governed_qc_input_count,
                        "contributor_stage_ids": report.governed_qc_contributor_stage_ids,
                        "contributor_tool_ids": report.governed_qc_contributor_tool_ids,
                        "raw_fastqc_dir": report.raw_fastqc_dir,
                        "trimmed_fastqc_dir": report.trimmed_fastqc_dir,
                        "multiqc_report": report.multiqc_report,
                        "multiqc_data": report.multiqc_data,
                        "multiqc_sample_count": report.multiqc_sample_count.or_else(|| multiqc_metrics.as_ref().map(|metrics| metrics.sample_count)),
                        "multiqc_module_count": report.multiqc_module_count.or_else(|| multiqc_metrics.as_ref().map(|metrics| metrics.module_count)),
                    });
                }
            }
        }
        if let Some(manifest_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "governed_qc_inputs_manifest")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_manifest) = fs::read_to_string(manifest_path) {
                if let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&raw_manifest) {
                    let contributor_entries = manifest
                        .get("contributors")
                        .and_then(serde_json::Value::as_array)
                        .cloned()
                        .unwrap_or_default();
                    let mut contributor_stage_ids = if contributor_entries.is_empty() {
                        manifest
                            .get("qc_inputs")
                            .and_then(serde_json::Value::as_array)
                            .into_iter()
                            .flatten()
                            .filter_map(|entry| {
                                entry.get("name").and_then(serde_json::Value::as_str)
                            })
                            .filter_map(parse_qc_contributor_identity)
                            .map(|(stage_id, _tool_id)| stage_id)
                            .collect::<Vec<_>>()
                    } else {
                        contributor_entries
                            .iter()
                            .filter_map(|entry| {
                                entry
                                    .get("stage_id")
                                    .and_then(serde_json::Value::as_str)
                                    .map(ToString::to_string)
                            })
                            .collect::<Vec<_>>()
                    };
                    contributor_stage_ids.sort();
                    contributor_stage_ids.dedup();
                    let mut contributor_tool_ids = if contributor_entries.is_empty() {
                        manifest
                            .get("qc_inputs")
                            .and_then(serde_json::Value::as_array)
                            .into_iter()
                            .flatten()
                            .filter_map(|entry| {
                                entry.get("name").and_then(serde_json::Value::as_str)
                            })
                            .filter_map(parse_qc_contributor_identity)
                            .map(|(_stage_id, tool_id)| tool_id)
                            .collect::<Vec<_>>()
                    } else {
                        contributor_entries
                            .iter()
                            .filter_map(|entry| {
                                entry
                                    .get("contributor_id")
                                    .and_then(serde_json::Value::as_str)
                                    .and_then(|contributor_id| {
                                        contributor_id
                                            .rsplit_once('.')
                                            .map(|(_, tool_id)| tool_id.to_string())
                                    })
                            })
                            .collect::<Vec<_>>()
                    };
                    contributor_tool_ids.sort();
                    contributor_tool_ids.dedup();
                    let contributor_count = if contributor_entries.is_empty() {
                        manifest
                            .get("qc_inputs")
                            .and_then(serde_json::Value::as_array)
                            .map_or(0, std::vec::Vec::len)
                    } else {
                        contributor_entries.len()
                    };
                    return serde_json::json!({
                        "lineage_hash": manifest.get("lineage_hash").cloned().unwrap_or(serde_json::Value::Null),
                        "contributor_artifact_count": contributor_count,
                        "contributor_stage_ids": contributor_stage_ids,
                        "contributor_tool_ids": contributor_tool_ids,
                        "raw_fastqc_dir": manifest.get("raw_fastqc_dir").cloned().unwrap_or(serde_json::Value::Null),
                        "multiqc_sample_count": multiqc_metrics.as_ref().map(|metrics| metrics.sample_count),
                        "multiqc_module_count": multiqc_metrics.as_ref().map(|metrics| metrics.module_count),
                    });
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.validate_reads" {
        if let Some(semantics) = validate_semantic_metrics(artifacts) {
            return semantics;
        }
    }
    if plan.stage_id.as_str() == "fastq.normalize_primers" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_normalize_primers_report(&raw_report) {
                    return serde_json::json!({
                        "paired_mode": report.paired_mode,
                        "primer_set_id": report.primer_set_id,
                        "marker_id": report.marker_id,
                        "orientation_policy": report.orientation_policy,
                        "max_mismatch_rate": report.max_mismatch_rate,
                        "min_overlap_bp": report.min_overlap_bp,
                        "reads_in": report.reads_in,
                        "reads_out": report.reads_out,
                        "primer_trimmed_reads": report.primer_trimmed_reads,
                        "primer_trimmed_fraction": report.primer_trimmed_fraction,
                        "orientation_forward_fraction": report.orientation_forward_fraction,
                        "primer_orientation_report": report.primer_orientation_report,
                        "primer_stats_json": report.primer_stats_json,
                        "raw_backend_report": report.raw_backend_report,
                        "raw_backend_report_format": report.raw_backend_report_format,
                    });
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.index_reference" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_index_reference_report(&raw_report) {
                    return serde_json::json!({
                        "threads": report.threads,
                        "index_format": report.index_format,
                        "reference_bytes": report.reference_bytes,
                        "index_bytes": report.index_bytes,
                        "index_file_count": report.index_file_count,
                        "index_prefix": report.index_prefix,
                        "emitted_file_count": report.emitted_files.len(),
                        "emitted_files": report.emitted_files,
                    });
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.normalize_abundance" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_normalize_abundance_report(&raw_report) {
                    return serde_json::json!({
                        "method": report.method,
                        "input_value_column": report.input_value_column,
                        "normalized_value_column": report.normalized_value_column,
                        "compositional_rule": report.compositional_rule,
                        "scale_factor": report.scale_factor,
                        "table_rows": report.table_rows,
                        "sample_count": report.sample_count,
                        "feature_count": report.feature_count,
                        "zero_fraction": report.zero_fraction,
                        "per_sample_sums": report.per_sample_sums,
                        "raw_backend_report": report.raw_backend_report,
                        "raw_backend_report_format": report.raw_backend_report_format,
                        "used_fallback": report.used_fallback,
                    });
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.infer_asvs" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_infer_asvs_report(&raw_report) {
                    return serde_json::json!({
                        "paired_mode": report.paired_mode,
                        "denoising_method": report.denoising_method,
                        "pooling_mode": report.pooling_mode,
                        "chimera_policy": report.chimera_policy,
                        "requires_r_runtime": report.requires_r_runtime,
                        "output_table_kind": report.output_table_kind,
                        "asv_count": report.asv_count,
                        "sample_count": report.sample_count,
                        "representative_sequence_count": report.representative_sequence_count,
                        "asv_table_tsv": report.asv_table_tsv,
                        "asv_sequences_fasta": report.asv_sequences_fasta,
                        "taxonomy_ready_fasta": report.taxonomy_ready_fasta,
                        "taxonomy_ready_fastq": report.taxonomy_ready_fastq,
                        "used_fallback": report.used_fallback,
                        "raw_backend_report": report.raw_backend_report,
                        "raw_backend_report_format": report.raw_backend_report_format,
                    });
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.cluster_otus" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_cluster_otus_report(&raw_report) {
                    return serde_json::json!({
                        "otu_identity": report.otu_identity,
                        "threads": report.threads,
                        "otu_count": report.otu_count,
                        "sample_count": report.sample_count,
                        "representative_sequence_count": report.representative_sequence_count,
                        "output_table_kind": report.output_table_kind,
                        "otu_table": report.otu_table,
                        "otu_representatives": report.otu_representatives,
                        "taxonomy_ready_fasta": report.taxonomy_ready_fasta,
                        "taxonomy_ready_fastq": report.taxonomy_ready_fastq,
                        "used_fallback": report.used_fallback,
                        "raw_backend_report": report.raw_backend_report,
                        "raw_backend_report_format": report.raw_backend_report_format,
                    });
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.profile_reads" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "qc_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_profile_reads_report(&raw_report) {
                    return serde_json::json!({
                        "paired_mode": report.paired_mode,
                        "threads": report.threads,
                        "reads_total": report.reads_total,
                        "bases_total": report.bases_total,
                        "mean_q": report.mean_q,
                        "gc_percent": report.gc_percent,
                        "length_histogram_source": report.length_histogram_source,
                        "length_histogram_bins": report.length_histogram.len(),
                        "mate_summary_count": report.mate_summaries.len(),
                        "mate_summaries": report.mate_summaries,
                        "qc_tsv": report.qc_tsv,
                        "qc_plots_dir": report.qc_plots_dir,
                        "raw_backend_report": report.raw_backend_report,
                        "raw_backend_report_format": report.raw_backend_report_format,
                    });
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.profile_read_lengths" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_profile_read_lengths_report(&raw_report) {
                    return serde_json::json!({
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
                    });
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.profile_overrepresented_sequences" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_profile_overrepresented_report(&raw_report) {
                    return serde_json::json!({
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
                    });
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.remove_duplicates" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_remove_duplicates_report(&raw_report) {
                    let provenance = artifacts
                        .iter()
                        .find(|artifact| artifact.name.as_str() == "duplicate_provenance_json")
                        .and_then(|artifact| fs::read_to_string(&artifact.path).ok())
                        .and_then(|raw| parse_remove_duplicates_provenance(&raw).ok());
                    return serde_json::json!({
                        "paired_mode": report.paired_mode,
                        "dedup_mode": report.dedup_mode,
                        "keep_order": report.keep_order,
                        "pair_count_match": report.pair_count_match,
                        "duplicates_removed": report.duplicates_removed,
                        "dedup_rate": report.dedup_rate,
                        "duplicate_class_count": report.duplicate_classes.len(),
                        "duplicate_classes": report.duplicate_classes,
                        "duplicate_provenance_json": report.duplicate_provenance_json,
                        "raw_backend_report": report.raw_backend_report,
                        "raw_backend_report_format": report.raw_backend_report_format,
                        "backend_log": provenance.as_ref().and_then(|value| value.backend_log.clone()),
                    });
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.remove_chimeras" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_remove_chimeras_report(&raw_report) {
                    return serde_json::json!({
                        "paired_mode": report.paired_mode,
                        "threads": report.threads,
                        "method": report.method,
                        "detection_scope": report.detection_scope,
                        "reads_in": report.reads_in,
                        "reads_out": report.reads_out,
                        "chimeras_removed": report.chimeras_removed,
                        "chimera_fraction": report.chimera_fraction,
                        "used_fallback": report.used_fallback,
                        "chimeras_fasta": report.chimeras_fasta,
                        "uchime_report_tsv": report.uchime_report_tsv,
                        "raw_backend_report": report.raw_backend_report,
                        "raw_backend_report_format": report.raw_backend_report_format,
                        "backend_metrics": report.backend_metrics,
                    });
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.detect_adapters" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_detect_adapters_report(&raw_report) {
                    return serde_json::json!({
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
                        "adapter_evidence_dir": report.adapter_evidence_dir,
                    });
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.trim_reads" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_trim_reads_report(&raw_report) {
                    let mut semantics = serde_json::Map::from_iter([
                        (
                            "paired_mode".to_string(),
                            serde_json::json!(report.paired_mode),
                        ),
                        ("threads".to_string(), serde_json::json!(report.threads)),
                        (
                            "min_length".to_string(),
                            serde_json::json!(report.min_length),
                        ),
                        (
                            "quality_cutoff".to_string(),
                            serde_json::json!(report.quality_cutoff),
                        ),
                        (
                            "adapter_policy".to_string(),
                            serde_json::json!(report.adapter_policy),
                        ),
                        (
                            "adapter_overrides".to_string(),
                            serde_json::json!(report.adapter_overrides),
                        ),
                        (
                            "polyx_policy".to_string(),
                            serde_json::json!(report.polyx_policy),
                        ),
                        ("n_policy".to_string(), serde_json::json!(report.n_policy)),
                        (
                            "contaminant_policy".to_string(),
                            serde_json::json!(report.contaminant_policy),
                        ),
                        (
                            "adapter_bank_id".to_string(),
                            serde_json::json!(report.adapter_bank_id),
                        ),
                        (
                            "polyx_bank_id".to_string(),
                            serde_json::json!(report.polyx_bank_id),
                        ),
                        (
                            "contaminant_bank_id".to_string(),
                            serde_json::json!(report.contaminant_bank_id),
                        ),
                        ("reads_in".to_string(), serde_json::json!(report.reads_in)),
                        ("reads_out".to_string(), serde_json::json!(report.reads_out)),
                        ("bases_in".to_string(), serde_json::json!(report.bases_in)),
                        ("bases_out".to_string(), serde_json::json!(report.bases_out)),
                        ("pairs_in".to_string(), serde_json::json!(report.pairs_in)),
                        ("pairs_out".to_string(), serde_json::json!(report.pairs_out)),
                        (
                            "mean_q_before".to_string(),
                            serde_json::json!(report.mean_q_before),
                        ),
                        (
                            "mean_q_after".to_string(),
                            serde_json::json!(report.mean_q_after),
                        ),
                        (
                            "raw_backend_report_format".to_string(),
                            serde_json::json!(report.raw_backend_report_format),
                        ),
                    ]);
                    if let (Some(raw_backend_report), Some(raw_backend_report_format)) = (
                        report.raw_backend_report.as_deref(),
                        report.raw_backend_report_format.as_deref(),
                    ) {
                        if let Ok(raw_backend_payload) = fs::read_to_string(raw_backend_report) {
                            match raw_backend_report_format {
                                "fastp_json" => {
                                    if let Ok(metrics) = parse_fastp_metrics(&raw_backend_payload) {
                                        semantics.insert(
                                            "passed_filter_reads".to_string(),
                                            serde_json::json!(metrics.passed_filter_reads),
                                        );
                                        semantics.insert(
                                            "low_quality_reads".to_string(),
                                            serde_json::json!(metrics.low_quality_reads),
                                        );
                                        semantics.insert(
                                            "too_many_n_reads".to_string(),
                                            serde_json::json!(metrics.too_many_n_reads),
                                        );
                                        semantics.insert(
                                            "too_short_reads".to_string(),
                                            serde_json::json!(metrics.too_short_reads),
                                        );
                                    }
                                }
                                "bbduk_stats" => {
                                    if let Ok(reads_removed) =
                                        parse_bbduk_reads_removed(&raw_backend_payload)
                                    {
                                        semantics.insert(
                                            "reads_removed".to_string(),
                                            serde_json::json!(reads_removed),
                                        );
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    return serde_json::Value::Object(semantics);
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.filter_low_complexity" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "filter_report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_filter_low_complexity_report(&raw_report) {
                    let mut semantics = serde_json::Map::from_iter([
                        (
                            "paired_mode".to_string(),
                            serde_json::json!(report.paired_mode),
                        ),
                        ("threads".to_string(), serde_json::json!(report.threads)),
                        (
                            "entropy_threshold".to_string(),
                            serde_json::json!(report.entropy_threshold),
                        ),
                        (
                            "polyx_threshold".to_string(),
                            serde_json::json!(report.polyx_threshold),
                        ),
                        ("reads_in".to_string(), serde_json::json!(report.reads_in)),
                        ("reads_out".to_string(), serde_json::json!(report.reads_out)),
                        (
                            "reads_removed_low_complexity".to_string(),
                            serde_json::json!(report.reads_removed_low_complexity),
                        ),
                        ("bases_in".to_string(), serde_json::json!(report.bases_in)),
                        ("bases_out".to_string(), serde_json::json!(report.bases_out)),
                        (
                            "mean_q_before".to_string(),
                            serde_json::json!(report.mean_q_before),
                        ),
                        (
                            "mean_q_after".to_string(),
                            serde_json::json!(report.mean_q_after),
                        ),
                        (
                            "raw_backend_report_format".to_string(),
                            serde_json::json!(report.raw_backend_report_format),
                        ),
                    ]);
                    if let Some(backend_metrics) = report
                        .backend_metrics
                        .as_ref()
                        .and_then(serde_json::Value::as_object)
                    {
                        for (metric_name, metric_value) in backend_metrics {
                            semantics.insert(metric_name.clone(), metric_value.clone());
                        }
                    }
                    return serde_json::Value::Object(semantics);
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.extract_umis" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_extract_umis_report(&raw_report) {
                    let mut semantics = serde_json::Map::from_iter([
                        (
                            "paired_mode".to_string(),
                            serde_json::json!(report.paired_mode),
                        ),
                        ("threads".to_string(), serde_json::json!(report.threads)),
                        (
                            "umi_pattern".to_string(),
                            serde_json::json!(report.umi_pattern),
                        ),
                        ("reads_in".to_string(), serde_json::json!(report.reads_in)),
                        ("reads_out".to_string(), serde_json::json!(report.reads_out)),
                        ("bases_in".to_string(), serde_json::json!(report.bases_in)),
                        ("bases_out".to_string(), serde_json::json!(report.bases_out)),
                        ("pairs_in".to_string(), serde_json::json!(report.pairs_in)),
                        ("pairs_out".to_string(), serde_json::json!(report.pairs_out)),
                        (
                            "reads_with_umi".to_string(),
                            serde_json::json!(report.reads_with_umi),
                        ),
                        (
                            "mean_q_before".to_string(),
                            serde_json::json!(report.mean_q_before),
                        ),
                        (
                            "mean_q_after".to_string(),
                            serde_json::json!(report.mean_q_after),
                        ),
                        (
                            "raw_backend_report_format".to_string(),
                            serde_json::json!(report.raw_backend_report_format),
                        ),
                    ]);
                    if let Some(backend_metrics) = report
                        .backend_metrics
                        .as_ref()
                        .and_then(serde_json::Value::as_object)
                    {
                        for (metric_name, metric_value) in backend_metrics {
                            semantics.insert(metric_name.clone(), metric_value.clone());
                        }
                    }
                    return serde_json::Value::Object(semantics);
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.filter_reads" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_filter_reads_report(&raw_report) {
                    let mut semantics = serde_json::Map::from_iter([
                        (
                            "paired_mode".to_string(),
                            serde_json::json!(report.paired_mode),
                        ),
                        ("threads".to_string(), serde_json::json!(report.threads)),
                        ("max_n".to_string(), serde_json::json!(report.max_n)),
                        (
                            "max_n_fraction".to_string(),
                            serde_json::json!(report.max_n_fraction),
                        ),
                        (
                            "max_n_count".to_string(),
                            serde_json::json!(report.max_n_count),
                        ),
                        (
                            "low_complexity_threshold".to_string(),
                            serde_json::json!(report.low_complexity_threshold),
                        ),
                        (
                            "entropy_threshold".to_string(),
                            serde_json::json!(report.entropy_threshold),
                        ),
                        ("n_policy".to_string(), serde_json::json!(report.n_policy)),
                        (
                            "polyx_policy".to_string(),
                            serde_json::json!(report.polyx_policy),
                        ),
                        (
                            "contaminant_db".to_string(),
                            serde_json::json!(report.contaminant_db),
                        ),
                        ("reads_in".to_string(), serde_json::json!(report.reads_in)),
                        ("reads_out".to_string(), serde_json::json!(report.reads_out)),
                        (
                            "reads_dropped".to_string(),
                            serde_json::json!(report.reads_dropped),
                        ),
                        (
                            "reads_removed_by_n".to_string(),
                            serde_json::json!(report.reads_removed_by_n),
                        ),
                        (
                            "reads_removed_by_entropy".to_string(),
                            serde_json::json!(report.reads_removed_by_entropy),
                        ),
                        (
                            "reads_removed_low_complexity".to_string(),
                            serde_json::json!(report.reads_removed_low_complexity),
                        ),
                        (
                            "reads_removed_by_kmer".to_string(),
                            serde_json::json!(report.reads_removed_by_kmer),
                        ),
                        (
                            "reads_removed_contaminant_kmer".to_string(),
                            serde_json::json!(report.reads_removed_contaminant_kmer),
                        ),
                        (
                            "reads_removed_by_length".to_string(),
                            serde_json::json!(report.reads_removed_by_length),
                        ),
                        ("bases_in".to_string(), serde_json::json!(report.bases_in)),
                        ("bases_out".to_string(), serde_json::json!(report.bases_out)),
                        ("pairs_in".to_string(), serde_json::json!(report.pairs_in)),
                        ("pairs_out".to_string(), serde_json::json!(report.pairs_out)),
                        (
                            "mean_q_before".to_string(),
                            serde_json::json!(report.mean_q_before),
                        ),
                        (
                            "mean_q_after".to_string(),
                            serde_json::json!(report.mean_q_after),
                        ),
                        (
                            "raw_backend_report_format".to_string(),
                            serde_json::json!(report.raw_backend_report_format),
                        ),
                    ]);
                    if let Some(backend_metrics) = report
                        .backend_metrics
                        .as_ref()
                        .and_then(serde_json::Value::as_object)
                    {
                        for (metric_name, metric_value) in backend_metrics {
                            if metric_name == "schema_version" {
                                continue;
                            }
                            semantics.insert(metric_name.clone(), metric_value.clone());
                        }
                        return serde_json::Value::Object(semantics);
                    }
                    if let (Some(raw_backend_report), Some(raw_backend_report_format)) = (
                        report.raw_backend_report.as_deref(),
                        report.raw_backend_report_format.as_deref(),
                    ) {
                        if let Ok(raw_backend_payload) = fs::read_to_string(raw_backend_report) {
                            match raw_backend_report_format {
                                "fastp_json" => {
                                    if let Ok(metrics) = parse_fastp_metrics(&raw_backend_payload) {
                                        semantics.insert(
                                            "passed_filter_reads".to_string(),
                                            serde_json::json!(metrics.passed_filter_reads),
                                        );
                                        semantics.insert(
                                            "low_quality_reads".to_string(),
                                            serde_json::json!(metrics.low_quality_reads),
                                        );
                                        semantics.insert(
                                            "too_many_n_reads".to_string(),
                                            serde_json::json!(metrics.too_many_n_reads),
                                        );
                                        semantics.insert(
                                            "too_short_reads".to_string(),
                                            serde_json::json!(metrics.too_short_reads),
                                        );
                                    }
                                }
                                "bbduk_stats" => {
                                    if let Ok(reads_removed) =
                                        parse_bbduk_reads_removed(&raw_backend_payload)
                                    {
                                        semantics.insert(
                                            "reads_removed".to_string(),
                                            serde_json::json!(reads_removed),
                                        );
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    return serde_json::Value::Object(semantics);
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.remove_duplicates" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                let parsed: Result<(u64, u64), _> = parse_deduplicate_report(&raw_report);
                if let Ok((reads_in, reads_out)) = parsed {
                    let duplicates_removed = reads_in.saturating_sub(reads_out);
                    let dedup_rate = if reads_in > 0 {
                        duplicates_removed as f64 / reads_in as f64
                    } else {
                        0.0
                    };
                    return serde_json::json!({
                        "reads_in": reads_in,
                        "reads_out": reads_out,
                        "duplicates_removed": duplicates_removed,
                        "dedup_rate": dedup_rate,
                    });
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.trim_terminal_damage" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_terminal_damage_report(&raw_report) {
                    return serde_json::json!({
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
                        "raw_backend_report_format": report.raw_backend_report_format,
                    });
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.trim_polyg_tails" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_trim_polyg_report(&raw_report) {
                    let mut semantics = serde_json::Map::from_iter([
                        (
                            "paired_mode".to_string(),
                            serde_json::json!(report.paired_mode),
                        ),
                        ("threads".to_string(), serde_json::json!(report.threads)),
                        (
                            "trim_polyg".to_string(),
                            serde_json::json!(report.trim_polyg),
                        ),
                        (
                            "min_polyg_run".to_string(),
                            serde_json::json!(report.min_polyg_run),
                        ),
                        (
                            "bases_trimmed_polyg".to_string(),
                            serde_json::json!(report.bases_trimmed_polyg),
                        ),
                        (
                            "polyx_bank_id".to_string(),
                            serde_json::json!(report.polyx_bank_id),
                        ),
                        (
                            "polyx_bank_hash".to_string(),
                            serde_json::json!(report.polyx_bank_hash),
                        ),
                        (
                            "polyx_preset".to_string(),
                            serde_json::json!(report.polyx_preset),
                        ),
                        (
                            "raw_backend_report".to_string(),
                            serde_json::json!(report.raw_backend_report),
                        ),
                        (
                            "raw_backend_report_format".to_string(),
                            serde_json::json!(report.raw_backend_report_format),
                        ),
                    ]);
                    if let Some(backend_metrics) = report
                        .backend_metrics
                        .as_ref()
                        .and_then(serde_json::Value::as_object)
                    {
                        for (metric_name, metric_value) in backend_metrics {
                            if metric_name == "schema_version" {
                                continue;
                            }
                            semantics.insert(metric_name.clone(), metric_value.clone());
                        }
                        return serde_json::Value::Object(semantics);
                    }
                    if let (Some(raw_backend_report), Some(raw_backend_report_format)) = (
                        report.raw_backend_report.as_deref(),
                        report.raw_backend_report_format.as_deref(),
                    ) {
                        if let Ok(raw_backend_report) = fs::read_to_string(raw_backend_report) {
                            match raw_backend_report_format {
                                "fastp_json" => {
                                    if let Ok(metrics) = parse_fastp_metrics(&raw_backend_report) {
                                        semantics.insert(
                                            "passed_filter_reads".to_string(),
                                            serde_json::json!(metrics.passed_filter_reads),
                                        );
                                        semantics.insert(
                                            "low_quality_reads".to_string(),
                                            serde_json::json!(metrics.low_quality_reads),
                                        );
                                        semantics.insert(
                                            "too_many_n_reads".to_string(),
                                            serde_json::json!(metrics.too_many_n_reads),
                                        );
                                        semantics.insert(
                                            "too_short_reads".to_string(),
                                            serde_json::json!(metrics.too_short_reads),
                                        );
                                    }
                                }
                                "bbduk_stats" => {
                                    if let Ok(reads_removed) =
                                        parse_bbduk_reads_removed(&raw_backend_report)
                                    {
                                        semantics.insert(
                                            "reads_removed".to_string(),
                                            serde_json::json!(reads_removed),
                                        );
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    return serde_json::Value::Object(semantics);
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.correct_errors" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_correct_errors_report(&raw_report) {
                    return serde_json::json!({
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
                        "kmer_fix_rate": report.kmer_fix_rate,
                        "correction_effect": report.correction_effect,
                        "raw_backend_report": report.raw_backend_report,
                        "raw_backend_report_format": report.raw_backend_report_format,
                        "input_r1": report.input_r1,
                        "input_r2": report.input_r2,
                        "output_r1": report.output_r1,
                        "output_r2": report.output_r2,
                    });
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.screen_taxonomy" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "classification_report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_screen_taxonomy_report(&raw_report) {
                    return serde_json::json!({
                        "paired_mode": report.paired_mode,
                        "classifier": report.classifier,
                        "report_format": report.report_format,
                        "assignment_format": report.assignment_format,
                        "database_catalog_id": report.database_catalog_id,
                        "database_artifact_id": report.database_artifact_id,
                        "database_digest": report.database_digest,
                        "minimum_confidence": report.minimum_confidence,
                        "emit_unclassified": report.emit_unclassified,
                        "contamination_rate": report.contamination_rate,
                        "classified_fraction": report.classified_fraction,
                        "unclassified_fraction": report.unclassified_fraction,
                        "summary_entry_count": report.summary_entries.len(),
                        "top_taxa": report.top_taxa.iter().map(|entry| entry.label.clone()).collect::<Vec<_>>(),
                    });
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.deplete_rrna" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "rrna_report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_deplete_rrna_report(&raw_report) {
                    return serde_json::json!({
                        "paired_mode": report.paired_mode,
                        "threads": report.threads,
                        "rrna_db": report.rrna_db,
                        "database_artifact_id": report.database_artifact_id,
                        "database_build_id": report.database_build_id,
                        "screening_engine": report.screening_engine,
                        "report_format": report.report_format,
                        "min_identity": report.min_identity,
                        "reads_removed": report.reads_removed,
                        "bases_removed": report.bases_removed,
                        "rrna_fraction_removed": report.rrna_fraction_removed,
                        "rrna_report_tsv": report.rrna_report_tsv,
                        "raw_backend_report": report.raw_backend_report,
                        "raw_backend_report_format": report.raw_backend_report_format,
                    });
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.deplete_reference_contaminants" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "contaminant_screen_report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_deplete_reference_contaminants_report(&raw_report) {
                    return serde_json::json!({
                        "paired_mode": report.paired_mode,
                        "threads": report.threads,
                        "reference_catalog_id": report.reference_catalog_id,
                        "contaminant_reference": report.contaminant_reference,
                        "index_artifact": report.index_artifact,
                        "reference_index_backend": report.reference_index_backend,
                        "reference_build_id": report.reference_build_id,
                        "reference_digest": report.reference_digest,
                        "retain_unmapped_pairs": report.retain_unmapped_pairs,
                        "reads_removed": report.reads_removed,
                        "bases_removed": report.bases_removed,
                        "contaminant_fraction_removed": report.contaminant_fraction_removed,
                        "raw_backend_report": report.raw_backend_report,
                        "raw_backend_report_format": report.raw_backend_report_format,
                    });
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.deplete_host" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "host_depletion_report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_deplete_host_report(&raw_report) {
                    return serde_json::json!({
                        "paired_mode": report.paired_mode,
                        "threads": report.threads,
                        "reference_scope": report.reference_scope,
                        "reference_catalog_id": report.reference_catalog_id,
                        "reference_index_artifact_id": report.reference_index_artifact_id,
                        "reference_index_backend": report.reference_index_backend,
                        "reference_build_id": report.reference_build_id,
                        "reference_digest": report.reference_digest,
                        "identity_threshold": report.identity_threshold,
                        "retained_read_policy": report.retained_read_policy,
                        "report_format": report.report_format,
                        "retain_unmapped_pairs": report.retain_unmapped_pairs,
                        "reads_removed": report.reads_removed,
                        "bases_removed": report.bases_removed,
                        "host_fraction_removed": report.host_fraction_removed,
                        "removed_host_r1": report.removed_host_r1,
                        "removed_host_r2": report.removed_host_r2,
                        "raw_backend_report": report.raw_backend_report,
                        "raw_backend_report_format": report.raw_backend_report_format,
                    });
                }
            }
        }
    }
    serde_json::Value::Null
}

pub(super) fn validate_semantic_metrics(
    artifacts: &[ArtifactRef],
) -> Option<serde_json::Value> {
    let report = artifacts
        .iter()
        .find(|artifact| artifact.name.as_str() == "validation_report")
        .map(|artifact| artifact.path.as_path())
        .and_then(|report_path| {
            fs::read_to_string(report_path)
                .ok()
                .and_then(|raw_report| parse_validation_report(&raw_report).ok())
        });
    let manifest = artifacts
        .iter()
        .find(|artifact| artifact.name.as_str() == "validated_reads_manifest")
        .map(|artifact| artifact.path.as_path())
        .and_then(|manifest_path| {
            fs::read_to_string(manifest_path)
                .ok()
                .and_then(|raw_manifest| parse_validated_reads_manifest(&raw_manifest).ok())
        });
    if report.is_none() && manifest.is_none() {
        return None;
    }
    Some(serde_json::json!({
        "tool_id": report.as_ref().map(|value| value.tool_id.clone()).or_else(|| manifest.as_ref().map(|value| value.tool_id.clone())),
        "validation_mode": report.as_ref().and_then(|value| serde_json::to_value(&value.validation_mode).ok()).unwrap_or(serde_json::Value::Null),
        "pair_sync_policy": report.as_ref().and_then(|value| serde_json::to_value(&value.pair_sync_policy).ok()).unwrap_or(serde_json::Value::Null),
        "failure_class": report.as_ref().and_then(|value| serde_json::to_value(&value.failure_class).ok()).unwrap_or(serde_json::Value::Null),
        "strict_pass": report.as_ref().map(|value| serde_json::json!(value.strict_pass)).unwrap_or(serde_json::Value::Null),
        "exit_code": report.as_ref().map(|value| serde_json::json!(value.exit_code)).unwrap_or(serde_json::Value::Null),
        "validated_inputs": report.as_ref().map(|value| serde_json::json!(value.validated_inputs)).unwrap_or(serde_json::Value::Null),
        "validated_reads_r1": report.as_ref().map(|value| serde_json::json!(value.validated_reads_r1)).unwrap_or(serde_json::Value::Null),
        "validated_reads_r2": report.as_ref().and_then(|value| serde_json::to_value(value.validated_reads_r2).ok()).unwrap_or(serde_json::Value::Null),
        "validated_pairs": report.as_ref().and_then(|value| serde_json::to_value(value.validated_pairs).ok()).unwrap_or(serde_json::Value::Null),
        "status_r1": report.as_ref().map(|value| serde_json::json!(value.status_r1)).unwrap_or(serde_json::Value::Null),
        "status_r2": report.as_ref().map(|value| serde_json::json!(value.status_r2)).unwrap_or(serde_json::Value::Null),
        "pair_sync_checked": report.as_ref().map(|value| serde_json::json!(value.pair_sync_checked)).or_else(|| manifest.as_ref().map(|value| serde_json::json!(value.pair_sync_checked))).unwrap_or(serde_json::Value::Null),
        "pair_sync_pass": report.as_ref().and_then(|value| serde_json::to_value(value.pair_sync_pass).ok()).or_else(|| manifest.as_ref().and_then(|value| serde_json::to_value(value.pair_sync_pass).ok())).unwrap_or(serde_json::Value::Null),
        "pair_count_match": report.as_ref().and_then(|value| serde_json::to_value(value.pair_count_match).ok()).unwrap_or(serde_json::Value::Null),
        "paired_mode": manifest.as_ref().and_then(|value| serde_json::to_value(value.paired_mode).ok()).unwrap_or(serde_json::Value::Null),
        "validated_stream_ids": manifest.as_ref().map(|value| serde_json::json!(value.validated_stream_ids)).unwrap_or(serde_json::Value::Null),
        "validation_report": manifest.as_ref().map(|value| serde_json::json!(value.validation_report)).unwrap_or(serde_json::Value::Null),
    }))
}

fn parse_qc_contributor_identity(name: &str) -> Option<(String, String)> {
    let mut parts = name.split('.');
    let domain = parts.next()?;
    let stage = parts.next()?;
    let tool = parts.next()?;
    Some((format!("{domain}.{stage}"), tool.to_string()))
}
