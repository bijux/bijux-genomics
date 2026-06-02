#![allow(clippy::too_many_lines)]

use super::{Context, Result, StageResultV1};

mod standardized_metrics;

use self::standardized_metrics::discover_screen_taxonomy_report_path;

pub(super) fn emit_fastq_stage_extra_artifacts(
    stage_root: &std::path::Path,
    stage_id: &str,
    execution: &StageResultV1,
) -> Result<()> {
    let payload = match stage_id {
        "fastq.index_reference" => {
            let report_path = execution
                .outputs
                .iter()
                .find(|path| {
                    path.file_name().and_then(|name| name.to_str())
                        == Some("index_reference_report.json")
                })
                .cloned()
                .unwrap_or_else(|| stage_root.join("index_reference_report.json"));
            let governed = std::fs::read_to_string(&report_path).ok().and_then(|raw| {
                bijux_dna_domain_fastq::observer::parse_index_reference_report(&raw).ok()
            });
            Some(serde_json::json!({
                "schema_version": "bijux.fastq.index_reference.extra_artifacts.v2",
                "stage": stage_id,
                "tool": governed.as_ref().map(|report| report.tool_id.clone()),
                "threads": governed.as_ref().map(|report| report.threads),
                "index_format": governed.as_ref().map(|report| report.index_format.clone()),
                "reference_index": governed.as_ref().map(|report| report.reference_index.clone()),
                "index_prefix": governed.as_ref().and_then(|report| report.index_prefix.clone()),
                "emitted_file_count": governed.as_ref().map(|report| report.emitted_files.len()),
                "index_bytes": governed.as_ref().map(|report| report.index_bytes),
                "report_json": report_path,
            }))
        }
        "fastq.detect_adapters" => {
            let report_path = stage_root.join("adapter_report.json");
            let governed = std::fs::read_to_string(&report_path).ok().and_then(|raw| {
                bijux_dna_domain_fastq::observer::parse_detect_adapters_report(&raw).ok()
            });
            Some(serde_json::json!({
                "schema_version": "bijux.fastq.detect_adapters.extra_artifacts.v2",
                "stage": stage_id,
                "tool": governed.as_ref().map(|report| report.tool_id.clone()),
                "paired_mode": governed.as_ref().map(|report| report.paired_mode),
                "threads": governed.as_ref().map(|report| report.threads),
                "inspection_mode": governed.as_ref().map(|report| report.inspection_mode.clone()),
                "evidence_engine": governed.as_ref().map(|report| report.evidence_engine.clone()),
                "evidence_scope": governed.as_ref().map(|report| report.evidence_scope.clone()),
                "evidence_format": governed.as_ref().map(|report| report.evidence_format.clone()),
                "adapter_report": governed.as_ref().map(|report| report.report_json.clone()),
                "candidate_adapter_count": governed.as_ref().map(|report| report.candidate_adapter_count),
                "detected_adapter_ids": governed.as_ref().map(|report| report.detected_adapter_ids.clone()),
                "detection_confidence": governed.as_ref().and_then(|report| report.detection_confidence),
                "detection_threshold": governed.as_ref().and_then(|report| report.detection_threshold),
                "adapter_trimmed_fraction": governed.as_ref().and_then(|report| report.adapter_trimmed_fraction),
                "adapter_evidence_dir": governed.as_ref().map(|report| report.adapter_evidence_dir.clone()),
                "report_json": report_path,
            }))
        }
        "fastq.detect_duplicates_premerge" => {
            let report_path = execution
                .outputs
                .iter()
                .find(|path| {
                    path.file_name().and_then(|name| name.to_str())
                        == Some("duplicate_signal_report.json")
                })
                .cloned()
                .unwrap_or_else(|| stage_root.join("duplicate_signal_report.json"));
            let governed = std::fs::read_to_string(&report_path).ok().and_then(|raw| {
                bijux_dna_domain_fastq::observer::parse_detect_duplicates_premerge_report(&raw).ok()
            });
            Some(serde_json::json!({
                "schema_version": "bijux.fastq.detect_duplicates_premerge.extra_artifacts.v1",
                "stage": stage_id,
                "tool": governed.as_ref().map(|report| report.tool_id.clone()),
                "paired_mode": governed.as_ref().map(|report| report.paired_mode),
                "duplicate_detection_policy": governed.as_ref().map(|report| report.duplicate_detection_policy.clone()),
                "measurement_scope": governed.as_ref().map(|report| report.measurement_scope.clone()),
                "modifies_reads": governed.as_ref().map(|report| report.modifies_reads),
                "advisory_only": governed.as_ref().map(|report| report.advisory_only),
                "reads_in": governed.as_ref().map(|report| report.reads_in),
                "duplicate_count": governed.as_ref().map(|report| report.duplicate_signal_reads),
                "duplicate_fraction": governed.as_ref().map(|report| report.duplicate_signal_fraction),
                "inspected_pair_count": governed.as_ref().and_then(|report| report.compared_read_pairs),
                "report_json": report_path,
            }))
        }
        "fastq.estimate_library_complexity_prealign" => {
            let report_path = execution
                .outputs
                .iter()
                .find(|path| {
                    path.file_name().and_then(|name| name.to_str())
                        == Some("library_complexity_report.json")
                })
                .cloned()
                .unwrap_or_else(|| stage_root.join("library_complexity_report.json"));
            let governed = std::fs::read_to_string(&report_path).ok().and_then(|raw| {
                bijux_dna_domain_fastq::observer::parse_estimate_library_complexity_prealign_report(
                    &raw,
                )
                .ok()
            });
            Some(serde_json::json!({
                "schema_version": "bijux.fastq.estimate_library_complexity_prealign.extra_artifacts.v1",
                "stage": stage_id,
                "tool": governed.as_ref().map(|report| report.tool_id.clone()),
                "paired_mode": governed.as_ref().map(|report| report.paired_mode),
                "complexity_policy": governed.as_ref().map(|report| report.complexity_policy.clone()),
                "estimate_method": governed.as_ref().map(|report| report.estimate_method.clone()),
                "modifies_reads": governed.as_ref().map(|report| report.modifies_reads),
                "advisory_only": governed.as_ref().map(|report| report.advisory_only),
                "reads_in": governed.as_ref().map(|report| report.reads_in),
                "estimated_complexity": governed.as_ref().and_then(|report| {
                    if report.insufficient_data_reason.is_some() {
                        None
                    } else {
                        Some(report.estimated_unique_fraction)
                    }
                }),
                "estimated_unique_fraction": governed.as_ref().map(|report| report.estimated_unique_fraction),
                "estimated_duplicate_fraction": governed.as_ref().map(|report| report.estimated_duplicate_fraction),
                "insufficient_data_reason": governed.as_ref().and_then(|report| report.insufficient_data_reason.clone()),
                "complexity_status": governed.as_ref().map(|report| {
                    if report.insufficient_data_reason.is_some() {
                        "insufficient_data"
                    } else {
                        "complexity_estimated"
                    }
                }),
                "kmer_size": governed.as_ref().and_then(|report| report.kmer_size),
                "report_json": report_path,
            }))
        }
        "fastq.trim_reads" => {
            let report_path = execution
                .outputs
                .iter()
                .find(|path| {
                    path.file_name().and_then(|name| name.to_str()) == Some("trim_report.json")
                })
                .cloned()
                .unwrap_or_else(|| stage_root.join("trim_report.json"));
            let governed = std::fs::read_to_string(&report_path).ok().and_then(|raw| {
                bijux_dna_domain_fastq::observer::parse_trim_reads_report(&raw).ok()
            });
            Some(serde_json::json!({
                "schema_version": "bijux.fastq.trim_reads.extra_artifacts.v2",
                "stage": stage_id,
                "tool": governed.as_ref().map(|report| report.tool_id.clone()),
                "paired_mode": governed.as_ref().map(|report| report.paired_mode),
                "threads": governed.as_ref().map(|report| report.threads),
                "min_length": governed.as_ref().map(|report| report.min_length),
                "quality_cutoff": governed.as_ref().and_then(|report| report.quality_cutoff),
                "adapter_policy": governed.as_ref().map(|report| report.adapter_policy.clone()),
                "polyx_policy": governed.as_ref().and_then(|report| report.polyx_policy.clone()),
                "n_policy": governed.as_ref().and_then(|report| report.n_policy.clone()),
                "contaminant_policy": governed.as_ref().and_then(|report| report.contaminant_policy.clone()),
                "adapter_bank_id": governed.as_ref().and_then(|report| report.adapter_bank_id.clone()),
                "adapter_bank_hash": governed.as_ref().and_then(|report| report.adapter_bank_hash.clone()),
                "adapter_preset": governed.as_ref().and_then(|report| report.adapter_preset.clone()),
                "trimmed_reads_r1": governed.as_ref().map(|report| report.output_r1.clone()),
                "trimmed_reads_r2": governed.as_ref().and_then(|report| report.output_r2.clone()),
                "report_json": report_path,
                "reads_retained": governed.as_ref().and_then(|report| report.reads_out),
                "reads_dropped": governed.as_ref().and_then(|report| {
                    report.reads_in.zip(report.reads_out).map(|(reads_in, reads_out)| {
                        reads_in.saturating_sub(reads_out)
                    })
                }),
                "bases_removed": governed.as_ref().and_then(|report| {
                    report.bases_in.zip(report.bases_out).map(|(bases_in, bases_out)| {
                        bases_in.saturating_sub(bases_out)
                    })
                }),
                "raw_backend_report": governed.as_ref().and_then(|report| report.raw_backend_report.clone()),
                "raw_backend_report_format": governed.as_ref().and_then(|report| report.raw_backend_report_format.clone()),
            }))
        }
        "fastq.filter_reads" => {
            let report_path = execution
                .outputs
                .iter()
                .find(|path| {
                    path.file_name().and_then(|name| name.to_str()) == Some("filter_report.json")
                })
                .cloned()
                .unwrap_or_else(|| stage_root.join("filter_report.json"));
            let governed = std::fs::read_to_string(&report_path).ok().and_then(|raw| {
                bijux_dna_domain_fastq::observer::parse_filter_reads_report(&raw).ok()
            });
            Some(serde_json::json!({
                "schema_version": "bijux.fastq.filter_reads.extra_artifacts.v2",
                "stage": stage_id,
                "tool": governed.as_ref().map(|report| report.tool_id.clone()),
                "paired_mode": governed.as_ref().map(|report| report.paired_mode),
                "threads": governed.as_ref().map(|report| report.threads),
                "filtered_reads_r1": governed.as_ref().map(|report| report.output_r1.clone()),
                "filtered_reads_r2": governed.as_ref().and_then(|report| report.output_r2.clone()),
                "max_n": governed.as_ref().and_then(|report| report.max_n),
                "max_n_fraction": governed.as_ref().and_then(|report| report.max_n_fraction),
                "max_n_count": governed.as_ref().and_then(|report| report.max_n_count),
                "low_complexity_threshold": governed.as_ref().and_then(|report| report.low_complexity_threshold),
                "entropy_threshold": governed.as_ref().and_then(|report| report.entropy_threshold),
                "n_policy": governed.as_ref().and_then(|report| report.n_policy.clone()),
                "polyx_policy": governed.as_ref().and_then(|report| report.polyx_policy.clone()),
                "contaminant_db": governed.as_ref().and_then(|report| report.contaminant_db.clone()),
                "reads_retained": governed.as_ref().map(|report| report.reads_out),
                "reads_removed": governed.as_ref().map(|report| report.reads_dropped),
                "reads_removed_by_n": governed.as_ref().map(|report| report.reads_removed_by_n),
                "reads_removed_by_entropy": governed.as_ref().map(|report| report.reads_removed_by_entropy),
                "reads_removed_low_complexity": governed.as_ref().map(|report| report.reads_removed_low_complexity),
                "reads_removed_by_kmer": governed.as_ref().map(|report| report.reads_removed_by_kmer),
                "reads_removed_contaminant_kmer": governed.as_ref().map(|report| report.reads_removed_contaminant_kmer),
                "reads_removed_by_length": governed.as_ref().map(|report| report.reads_removed_by_length),
                "raw_backend_report": governed.as_ref().and_then(|report| report.raw_backend_report.clone()),
                "raw_backend_report_format": governed.as_ref().and_then(|report| report.raw_backend_report_format.clone()),
                "report_json": report_path,
            }))
        }
        "fastq.correct_errors" => {
            let report_path = execution
                .outputs
                .iter()
                .find(|path| {
                    path.file_name().and_then(|name| name.to_str()) == Some("correct_report.json")
                })
                .cloned()
                .unwrap_or_else(|| stage_root.join("correct_report.json"));
            let governed = std::fs::read_to_string(&report_path).ok().and_then(|raw| {
                bijux_dna_domain_fastq::observer::parse_correct_errors_report(&raw).ok()
            });
            Some(serde_json::json!({
                "schema_version": "bijux.fastq.correct_errors.extra_artifacts.v2",
                "stage": stage_id,
                "tool": governed.as_ref().map(|report| report.tool_id.clone()),
                "paired_mode": governed.as_ref().map(|report| report.paired_mode),
                "threads": governed.as_ref().map(|report| report.threads),
                "correction_engine": governed.as_ref().map(|report| report.correction_engine.clone()),
                "quality_encoding": governed.as_ref().map(|report| report.quality_encoding.clone()),
                "kmer_size": governed.as_ref().and_then(|report| report.kmer_size),
                "genome_size": governed.as_ref().and_then(|report| report.genome_size),
                "max_memory_gb": governed.as_ref().and_then(|report| report.max_memory_gb),
                "trusted_kmer_artifact": governed.as_ref().and_then(|report| report.trusted_kmer_artifact.clone()),
                "conservative_mode": governed.as_ref().map(|report| report.conservative_mode),
                "corrected_reads_r1": governed.as_ref().map(|report| report.output_r1.clone()),
                "corrected_reads_r2": governed.as_ref().and_then(|report| report.output_r2.clone()),
                "corrected_reads": governed.as_ref().and_then(|report| report.corrected_reads),
                "changed_reads": governed.as_ref().and_then(|report| report.changed_reads),
                "unchanged_reads": governed.as_ref().and_then(|report| report.unchanged_reads),
                "correction_effect": governed.as_ref().and_then(|report| report.correction_effect.clone()),
                "raw_backend_report": governed.as_ref().and_then(|report| report.raw_backend_report.clone()),
                "raw_backend_report_format": governed.as_ref().and_then(|report| report.raw_backend_report_format.clone()),
                "report_json": report_path,
            }))
        }
        "fastq.filter_low_complexity" => {
            let report_path = stage_root.join("low_complexity_report.json");
            let governed = std::fs::read_to_string(&report_path).ok().and_then(|raw| {
                bijux_dna_domain_fastq::observer::parse_filter_low_complexity_report(&raw).ok()
            });
            Some(serde_json::json!({
                "schema_version": "bijux.fastq.filter_low_complexity.extra_artifacts.v2",
                "stage": stage_id,
                "tool": governed.as_ref().map(|report| report.tool_id.clone()),
                "paired_mode": governed.as_ref().map(|report| report.paired_mode),
                "threads": governed.as_ref().map(|report| report.threads),
                "filtered_fastq_r1": governed.as_ref().map(|report| report.output_r1.clone()),
                "filtered_fastq_r2": governed.as_ref().and_then(|report| report.output_r2.clone()),
                "entropy_threshold": governed.as_ref().and_then(|report| report.entropy_threshold),
                "polyx_threshold": governed.as_ref().and_then(|report| report.polyx_threshold),
                "reads_removed_low_complexity": governed.as_ref().map(|report| report.reads_removed_low_complexity),
                "raw_backend_report": governed.as_ref().and_then(|report| report.raw_backend_report.clone()),
                "raw_backend_report_format": governed.as_ref().and_then(|report| report.raw_backend_report_format.clone()),
                "report_json": report_path,
            }))
        }
        "fastq.profile_reads" => {
            let report_path = execution
                .outputs
                .iter()
                .find(|path| path.file_name().and_then(|name| name.to_str()) == Some("qc.json"))
                .cloned()
                .unwrap_or_else(|| stage_root.join("qc.json"));
            let governed = std::fs::read_to_string(&report_path).ok().and_then(|raw| {
                bijux_dna_domain_fastq::observer::parse_profile_reads_report(&raw).ok()
            });
            Some(serde_json::json!({
                "schema_version": "bijux.fastq.profile_reads.extra_artifacts.v2",
                "stage": stage_id,
                "paired_mode": governed.as_ref().map(|report| report.paired_mode),
                "threads": governed.as_ref().map(|report| report.threads),
                "length_histogram_source": governed.as_ref().map(|report| report.length_histogram_source.clone()),
                "length_histogram_bins": governed.as_ref().map(|report| report.length_histogram.len()),
                "mate_summary_count": governed.as_ref().map(|report| report.mate_summaries.len()),
                "qc_tsv": governed.as_ref().map(|report| report.qc_tsv.clone()),
                "qc_plots_dir": governed.as_ref().and_then(|report| report.qc_plots_dir.clone()),
                "raw_backend_report": governed.as_ref().and_then(|report| report.raw_backend_report.clone()),
                "raw_backend_report_format": governed.as_ref().and_then(|report| report.raw_backend_report_format.clone()),
                "report_json": report_path,
            }))
        }
        "fastq.profile_read_lengths" => {
            let report_path = execution
                .outputs
                .iter()
                .find(|path| {
                    path.file_name().and_then(|name| name.to_str())
                        == Some("profile_read_lengths_report.json")
                })
                .cloned()
                .unwrap_or_else(|| stage_root.join("profile_read_lengths_report.json"));
            let governed = std::fs::read_to_string(&report_path).ok().and_then(|raw| {
                bijux_dna_domain_fastq::observer::parse_profile_read_lengths_report(&raw).ok()
            });
            Some(serde_json::json!({
                "schema_version": "bijux.fastq.profile_read_lengths.extra_artifacts.v2",
                "stage": stage_id,
                "paired_mode": governed.as_ref().map(|report| report.paired_mode),
                "histogram_bins": governed.as_ref().map(|report| report.histogram_bins),
                "histogram_entry_count": governed.as_ref().map(|report| report.histogram.len()),
                "min_read_length": governed.as_ref().map(|report| report.min_read_length),
                "median_read_length": governed.as_ref().map(|report| report.median_read_length),
                "length_distribution_tsv": governed.as_ref().map(|report| report.length_distribution_tsv.clone()),
                "length_distribution_json": governed.as_ref().map(|report| report.length_distribution_json.clone()),
                "raw_backend_report": governed.as_ref().and_then(|report| report.raw_backend_report.clone()),
                "raw_backend_report_format": governed.as_ref().and_then(|report| report.raw_backend_report_format.clone()),
                "report_json": report_path,
            }))
        }
        "fastq.profile_overrepresented_sequences" => {
            let report_path = execution
                .outputs
                .iter()
                .find(|path| {
                    path.file_name().and_then(|name| name.to_str())
                        == Some("overrepresented_report.json")
                })
                .cloned()
                .unwrap_or_else(|| stage_root.join("overrepresented_report.json"));
            let governed = std::fs::read_to_string(&report_path).ok().and_then(|raw| {
                bijux_dna_domain_fastq::observer::parse_profile_overrepresented_report(&raw).ok()
            });
            Some(serde_json::json!({
                "schema_version": "bijux.fastq.profile_overrepresented.extra_artifacts.v2",
                "stage": stage_id,
                "paired_mode": governed.as_ref().map(|report| report.paired_mode),
                "threads": governed.as_ref().map(|report| report.threads),
                "top_k": governed.as_ref().map(|report| report.top_k),
                "sequence_count": governed.as_ref().map(|report| report.sequence_count),
                "flagged_sequences": governed.as_ref().map(|report| report.flagged_sequences),
                "row_count": governed.as_ref().map(|report| report.rows.len()),
                "overrepresented_sequences_tsv": governed.as_ref().map(|report| report.overrepresented_sequences_tsv.clone()),
                "overrepresented_sequences_json": governed.as_ref().map(|report| report.overrepresented_sequences_json.clone()),
                "raw_backend_report": governed.as_ref().and_then(|report| report.raw_backend_report.clone()),
                "raw_backend_report_format": governed.as_ref().and_then(|report| report.raw_backend_report_format.clone()),
                "report_json": report_path,
            }))
        }
        "fastq.trim_polyg_tails" => {
            let report_path = execution
                .outputs
                .iter()
                .find(|path| {
                    path.file_name().and_then(|name| name.to_str())
                        == Some("trim_polyg_tails_report.json")
                })
                .cloned()
                .unwrap_or_else(|| stage_root.join("trim_polyg_tails_report.json"));
            let governed = std::fs::read_to_string(&report_path).ok().and_then(|raw| {
                bijux_dna_domain_fastq::observer::parse_trim_polyg_report(&raw).ok()
            });
            Some(serde_json::json!({
                "schema_version": "bijux.fastq.trim_polyg_tails.extra_artifacts.v2",
                "stage": stage_id,
                "tool": governed.as_ref().map(|report| report.tool_id.clone()),
                "paired_mode": governed.as_ref().map(|report| report.paired_mode),
                "threads": governed.as_ref().map(|report| report.threads),
                "trim_polyg": governed.as_ref().map(|report| report.trim_polyg),
                "min_polyg_run": governed.as_ref().map(|report| report.min_polyg_run),
                "trimmed_tail_count": governed.as_ref().and_then(|report| report.trimmed_tail_count),
                "bases_trimmed_polyg": governed.as_ref().and_then(|report| report.bases_trimmed_polyg),
                "polyx_bank_id": governed.as_ref().and_then(|report| report.polyx_bank_id.clone()),
                "polyx_bank_hash": governed.as_ref().and_then(|report| report.polyx_bank_hash.clone()),
                "polyx_preset": governed.as_ref().and_then(|report| report.polyx_preset.clone()),
                "raw_backend_report": governed.as_ref().and_then(|report| report.raw_backend_report.clone()),
                "raw_backend_report_format": governed.as_ref().and_then(|report| report.raw_backend_report_format.clone()),
                "report_json": report_path,
            }))
        }
        "fastq.merge_pairs" => {
            let report_path = execution
                .outputs
                .iter()
                .find(|path| {
                    path.file_name().and_then(|name| name.to_str()) == Some("merge_report.json")
                })
                .cloned()
                .unwrap_or_else(|| stage_root.join("merge_report.json"));
            let governed = std::fs::read_to_string(&report_path).ok().and_then(|raw| {
                bijux_dna_domain_fastq::observer::parse_merge_pairs_report(&raw).ok()
            });
            Some(serde_json::json!({
                "schema_version": "bijux.fastq.merge_pairs.extra_artifacts.v2",
                "stage": stage_id,
                "tool": governed.as_ref().map(|report| report.tool_id.clone()),
                "paired_mode": governed.as_ref().map(|report| report.paired_mode),
                "merge_engine": governed.as_ref().map(|report| report.merge_engine.clone()),
                "threads": governed.as_ref().map(|report| report.threads),
                "merge_overlap": governed.as_ref().and_then(|report| report.merge_overlap),
                "min_length": governed.as_ref().and_then(|report| report.min_len),
                "unmerged_read_policy": governed.as_ref().map(|report| report.unmerged_read_policy.clone()),
                "reads_r1": governed.as_ref().map(|report| report.reads_r1),
                "reads_r2": governed.as_ref().map(|report| report.reads_r2),
                "input_pair_count": governed.as_ref().map(|report| report.reads_r1.min(report.reads_r2)),
                "reads_merged": governed.as_ref().map(|report| report.reads_merged),
                "reads_unmerged": governed.as_ref().map(|report| report.reads_unmerged),
                "merged_pair_count": governed.as_ref().map(|report| report.reads_merged.min(report.reads_r1.min(report.reads_r2))),
                "unmerged_pair_count": governed.as_ref().map(|report| {
                    let input_pair_count = report.reads_r1.min(report.reads_r2);
                    let merged_pair_count = report.reads_merged.min(input_pair_count);
                    report.reads_unmerged.min(input_pair_count.saturating_sub(merged_pair_count))
                }),
                "discarded_pair_count": governed.as_ref().map(|report| {
                    let input_pair_count = report.reads_r1.min(report.reads_r2);
                    let merged_pair_count = report.reads_merged.min(input_pair_count);
                    let unmerged_pair_count =
                        report.reads_unmerged.min(input_pair_count.saturating_sub(merged_pair_count));
                    input_pair_count.saturating_sub(merged_pair_count + unmerged_pair_count)
                }),
                "merge_rate": governed.as_ref().map(|report| report.merge_rate),
                "merged_reads": governed.as_ref().map(|report| report.merged_reads.clone()),
                "unmerged_reads_r1": governed.as_ref().and_then(|report| report.unmerged_reads_r1.clone()),
                "unmerged_reads_r2": governed.as_ref().and_then(|report| report.unmerged_reads_r2.clone()),
                "raw_backend_report": governed.as_ref().and_then(|report| report.raw_backend_report.clone()),
                "raw_backend_report_format": governed.as_ref().and_then(|report| report.raw_backend_report_format.clone()),
                "report_json": report_path,
            }))
        }
        "fastq.extract_umis" => {
            let report_path = stage_root.join("umi_report.json");
            let governed = std::fs::read_to_string(&report_path).ok().and_then(|raw| {
                bijux_dna_domain_fastq::observer::parse_extract_umis_report(&raw).ok()
            });
            Some(serde_json::json!({
                "schema_version": "bijux.fastq.extract_umis.extra_artifacts.v2",
                "stage": stage_id,
                "tool": governed.as_ref().map(|report| report.tool_id.clone()),
                "paired_mode": governed.as_ref().map(|report| report.paired_mode),
                "threads": governed.as_ref().map(|report| report.threads),
                "umi_pattern": governed.as_ref().map(|report| report.umi_pattern.clone()),
                "extraction_location": governed.as_ref().map(|report| report.extraction_location.clone()),
                "read_name_transform": governed.as_ref().map(|report| report.read_name_transform.clone()),
                "tag_header_format": governed.as_ref().map(|report| report.read_name_transform.clone()),
                "failed_extraction_policy": governed.as_ref().map(|report| report.failed_extraction_policy.clone()),
                "downstream_propagation": governed.as_ref().map(|report| report.downstream_propagation.clone()),
                "grouping_policy": governed.as_ref().map(|report| report.grouping_policy.clone()),
                "downstream_dedup_policy": governed.as_ref().map(|report| report.downstream_dedup_policy.clone()),
                "umi_reads_r1": governed.as_ref().map(|report| report.output_r1.clone()),
                "umi_reads_r2": governed.as_ref().and_then(|report| report.output_r2.clone()),
                "reads_in": governed.as_ref().map(|report| report.reads_in),
                "reads_out": governed.as_ref().map(|report| report.reads_out),
                "pairs_in": governed.as_ref().and_then(|report| report.pairs_in),
                "pairs_out": governed.as_ref().and_then(|report| report.pairs_out),
                "reads_with_umi": governed.as_ref().map(|report| report.reads_with_umi),
                "failed_extractions": governed.as_ref().and_then(|report| report.failed_extractions),
                "extracted_umi_count": governed.as_ref().map(|report| report.reads_with_umi),
                "invalid_umi_count": governed.as_ref().and_then(|report| report.failed_extractions),
                "raw_backend_report": governed.as_ref().and_then(|report| report.raw_backend_report.clone()),
                "raw_backend_report_format": governed.as_ref().and_then(|report| report.raw_backend_report_format.clone()),
                "report_json": report_path,
            }))
        }
        "fastq.screen_taxonomy" => {
            let report_path = discover_screen_taxonomy_report_path(stage_root, &execution.outputs)
                .unwrap_or_else(|| stage_root.join("classification_report.json"));
            let governed = std::fs::read_to_string(&report_path).ok().and_then(|raw| {
                bijux_dna_domain_fastq::observer::parse_screen_taxonomy_report(&raw).ok()
            });
            Some(serde_json::json!({
                "schema_version": "bijux.fastq.screen_taxonomy.extra_artifacts.v2",
                "stage": stage_id,
                "classifier": governed.as_ref().map(|report| report.classifier.clone()),
                "report_format": governed.as_ref().map(|report| report.report_format.clone()),
                "database_catalog_id": governed.as_ref().map(|report| report.database_catalog_id.clone()),
                "database_artifact_id": governed.as_ref().map(|report| report.database_artifact_id.clone()),
                "database_digest": governed.as_ref().and_then(|report| report.database_digest.clone()),
                "classified_fraction": governed.as_ref().and_then(|report| report.classified_fraction),
                "unclassified_fraction": governed.as_ref().and_then(|report| report.unclassified_fraction),
                "top_taxa": governed.as_ref().map(|report| report.top_taxa.clone()),
            }))
        }
        "fastq.report_qc" => {
            let report_path = execution
                .outputs
                .iter()
                .find(|path| {
                    path.file_name().and_then(|name| name.to_str()) == Some("report_qc_report.json")
                })
                .cloned()
                .unwrap_or_else(|| stage_root.join("report_qc_report.json"));
            let governed = std::fs::read_to_string(&report_path).ok().and_then(|raw| {
                bijux_dna_domain_fastq::observer::parse_report_qc_report(&raw).ok()
            });
            Some(serde_json::json!({
                "schema_version": "bijux.fastq.report_qc.extra_artifacts.v2",
                "stage": stage_id,
                "aggregation_engine": governed.as_ref().map(|report| report.aggregation_engine.clone()),
                "aggregation_scope": governed.as_ref().map(|report| report.aggregation_scope.clone()),
                "paired_mode": governed.as_ref().map(|report| report.paired_mode),
                "reads_in": governed.as_ref().map(|report| report.reads_in),
                "reads_out": governed.as_ref().map(|report| report.reads_out),
                "bases_in": governed.as_ref().map(|report| report.bases_in),
                "bases_out": governed.as_ref().map(|report| report.bases_out),
                "pairs_in": governed.as_ref().and_then(|report| report.pairs_in),
                "pairs_out": governed.as_ref().and_then(|report| report.pairs_out),
                "mean_q": governed.as_ref().map(|report| report.mean_q),
                "contamination_rate": governed.as_ref().map(|report| report.contamination_rate),
                "adapter_content_max": governed.as_ref().and_then(|report| report.adapter_content_max),
                "adapter_content_mean": governed.as_ref().and_then(|report| report.adapter_content_mean),
                "duplication_rate": governed.as_ref().and_then(|report| report.duplication_rate),
                "n_rate": governed.as_ref().and_then(|report| report.n_rate),
                "kmer_warning_count": governed.as_ref().and_then(|report| report.kmer_warning_count),
                "overrepresented_sequence_count": governed.as_ref().and_then(|report| report.overrepresented_sequence_count),
                "governed_qc_input_count": governed.as_ref().map(|report| report.governed_qc_input_count),
                "governed_qc_contributor_stage_ids": governed.as_ref().map(|report| report.governed_qc_contributor_stage_ids.clone()),
                "governed_qc_contributor_tool_ids": governed.as_ref().map(|report| report.governed_qc_contributor_tool_ids.clone()),
                "governed_qc_contributors": governed.as_ref().map(|report| report.governed_qc_contributors.clone()),
                "governed_qc_lineage_hash": governed.as_ref().and_then(|report| report.governed_qc_lineage_hash.clone()),
                "governed_qc_inputs_manifest": governed.as_ref().and_then(|report| report.governed_qc_inputs_manifest.clone()),
                "multiqc_sample_count": governed.as_ref().and_then(|report| report.multiqc_sample_count),
                "multiqc_module_count": governed.as_ref().and_then(|report| report.multiqc_module_count),
                "raw_fastqc_dir": governed.as_ref().and_then(|report| report.raw_fastqc_dir.clone()),
                "trimmed_fastqc_dir": governed.as_ref().and_then(|report| report.trimmed_fastqc_dir.clone()),
                "multiqc_report": governed.as_ref().and_then(|report| report.multiqc_report.clone()),
                "multiqc_data": governed.as_ref().and_then(|report| report.multiqc_data.clone()),
                "report_json": report_path,
            }))
        }
        "fastq.remove_duplicates" => {
            let report_path = execution
                .outputs
                .iter()
                .find(|path| {
                    path.file_name().and_then(|name| name.to_str())
                        == Some("deduplicate_report.json")
                })
                .cloned()
                .unwrap_or_else(|| stage_root.join("deduplicate_report.json"));
            let governed = std::fs::read_to_string(&report_path).ok().and_then(|raw| {
                bijux_dna_domain_fastq::observer::parse_remove_duplicates_report(&raw).ok()
            });
            Some(serde_json::json!({
                "schema_version": "bijux.fastq.remove_duplicates.extra_artifacts.v2",
                "stage": stage_id,
                "tool": governed.as_ref().map(|report| report.tool_id.clone()),
                "paired_mode": governed.as_ref().map(|report| report.paired_mode),
                "threads": governed.as_ref().map(|report| report.threads),
                "dedup_mode": governed.as_ref().map(|report| report.dedup_mode.clone()),
                "keep_order": governed.as_ref().map(|report| report.keep_order),
                "reads_in": governed.as_ref().map(|report| report.reads_in),
                "reads_out": governed.as_ref().map(|report| report.reads_out),
                "input_reads": governed.as_ref().map(|report| report.reads_in),
                "duplicate_reads": governed.as_ref().map(|report| report.duplicates_removed),
                "unique_reads": governed.as_ref().map(|report| report.reads_out),
                "output_reads": governed.as_ref().map(|report| report.reads_out),
                "reads_in_r2": governed.as_ref().and_then(|report| report.reads_in_r2),
                "reads_out_r2": governed.as_ref().and_then(|report| report.reads_out_r2),
                "pairs_in": governed.as_ref().and_then(|report| report.pairs_in),
                "pairs_out": governed.as_ref().and_then(|report| report.pairs_out),
                "pair_count_match": governed.as_ref().and_then(|report| report.pair_count_match),
                "duplicates_removed": governed.as_ref().map(|report| report.duplicates_removed),
                "dedup_rate": governed.as_ref().map(|report| report.dedup_rate),
                "duplicate_classes_tsv": governed.as_ref().and_then(|report| report.duplicate_classes_tsv.clone()),
                "duplicate_provenance_json": governed.as_ref().and_then(|report| report.duplicate_provenance_json.clone()),
                "duplicate_classes": governed.as_ref().map(|report| report.duplicate_classes.clone()),
                "raw_backend_report": governed.as_ref().and_then(|report| report.raw_backend_report.clone()),
                "raw_backend_report_format": governed.as_ref().and_then(|report| report.raw_backend_report_format.clone()),
                "report_json": report_path,
            }))
        }
        "fastq.deplete_reference_contaminants" => {
            let report_path = stage_root.join("contaminant_screen_report.json");
            let governed = std::fs::read_to_string(&report_path).ok().and_then(|raw| {
                bijux_dna_domain_fastq::observer::parse_deplete_reference_contaminants_report(&raw)
                    .ok()
            });
            Some(serde_json::json!({
                "schema_version": "bijux.fastq.deplete_reference_contaminants.extra_artifacts.v2",
                "stage": stage_id,
                "tool": governed.as_ref().map(|report| report.tool_id.clone()),
                "paired_mode": governed.as_ref().map(|report| report.paired_mode),
                "threads": governed.as_ref().map(|report| report.threads),
                "reference_catalog_id": governed.as_ref().map(|report| report.reference_catalog_id.clone()),
                "contaminant_reference": governed.as_ref().map(|report| report.contaminant_reference.clone()),
                "reference_index_artifact_id": governed.as_ref().map(|report| report.reference_index_artifact_id.clone()),
                "reference_index_backend": governed.as_ref().map(|report| report.reference_index_backend.clone()),
                "reference_build_id": governed.as_ref().and_then(|report| report.reference_build_id.clone()),
                "reference_digest": governed.as_ref().and_then(|report| report.reference_digest.clone()),
                "match_threshold": governed.as_ref().and_then(|report| report.match_threshold),
                "retained_read_role": governed.as_ref().map(|report| report.retained_read_role.clone()),
                "rejected_read_role": governed.as_ref().map(|report| report.rejected_read_role.clone()),
                "retain_unmapped_pairs": governed.as_ref().map(|report| report.retain_unmapped_pairs),
                "reads_removed": governed.as_ref().map(|report| report.reads_removed),
                "bases_removed": governed.as_ref().map(|report| report.bases_removed),
                "contaminant_fraction_removed": governed.as_ref().map(|report| report.contaminant_fraction_removed),
                "raw_backend_report": governed.as_ref().and_then(|report| report.raw_backend_report.clone()),
                "raw_backend_report_format": governed.as_ref().and_then(|report| report.raw_backend_report_format.clone()),
                "report_json": report_path,
            }))
        }
        "fastq.deplete_rrna" => {
            let report_path = stage_root.join("rrna_report.json");
            let governed = std::fs::read_to_string(&report_path).ok().and_then(|raw| {
                bijux_dna_domain_fastq::observer::parse_deplete_rrna_report(&raw).ok()
            });
            Some(serde_json::json!({
                "schema_version": "bijux.fastq.deplete_rrna.extra_artifacts.v2",
                "stage": stage_id,
                "tool": governed.as_ref().map(|report| report.tool_id.clone()),
                "paired_mode": governed.as_ref().map(|report| report.paired_mode),
                "threads": governed.as_ref().map(|report| report.threads),
                "rrna_db": governed.as_ref().and_then(|report| report.rrna_db.clone()),
                "database_artifact_id": governed.as_ref().map(|report| report.database_artifact_id.clone()),
                "database_build_id": governed.as_ref().and_then(|report| report.database_build_id.clone()),
                "database_digest": governed.as_ref().and_then(|report| report.database_digest.clone()),
                "screening_engine": governed.as_ref().map(|report| report.screening_engine.clone()),
                "report_format": governed.as_ref().map(|report| report.report_format.clone()),
                "min_identity": governed.as_ref().and_then(|report| report.min_identity),
                "retained_read_role": governed.as_ref().map(|report| report.retained_read_role.clone()),
                "rejected_read_role": governed.as_ref().map(|report| report.rejected_read_role.clone()),
                "reads_removed": governed.as_ref().map(|report| report.reads_removed),
                "bases_removed": governed.as_ref().map(|report| report.bases_removed),
                "rrna_fraction_removed": governed.as_ref().map(|report| report.rrna_fraction_removed),
                "rrna_report_tsv": governed.as_ref().map(|report| report.rrna_report_tsv.clone()),
                "raw_backend_report": governed.as_ref().and_then(|report| report.raw_backend_report.clone()),
                "raw_backend_report_format": governed.as_ref().and_then(|report| report.raw_backend_report_format.clone()),
                "report_json": report_path,
            }))
        }
        "fastq.deplete_host" => {
            let report_path = stage_root.join("host_depletion_report.json");
            let governed = std::fs::read_to_string(&report_path).ok().and_then(|raw| {
                bijux_dna_domain_fastq::observer::parse_deplete_host_report(&raw).ok()
            });
            Some(serde_json::json!({
                "schema_version": "bijux.fastq.deplete_host.extra_artifacts.v2",
                "stage": stage_id,
                "tool": governed.as_ref().map(|report| report.tool_id.clone()),
                "paired_mode": governed.as_ref().map(|report| report.paired_mode),
                "threads": governed.as_ref().map(|report| report.threads),
                "reference_scope": governed
                    .as_ref()
                    .map(|report| report.reference_scope.clone()),
                "reference_catalog_id": governed.as_ref().map(|report| report.reference_catalog_id.clone()),
                "reference_index_artifact_id": governed.as_ref().map(|report| report.reference_index_artifact_id.clone()),
                "reference_index_backend": governed.as_ref().map(|report| report.reference_index_backend.clone()),
                "reference_build_id": governed.as_ref().and_then(|report| report.reference_build_id.clone()),
                "reference_digest": governed.as_ref().and_then(|report| report.reference_digest.clone()),
                "masking_policy": governed
                    .as_ref()
                    .map(|report| report.masking_policy.clone()),
                "decoy_policy": governed
                    .as_ref()
                    .map(|report| report.decoy_policy.clone()),
                "decoy_catalog_id": governed.as_ref().and_then(|report| report.decoy_catalog_id.clone()),
                "identity_threshold": governed.as_ref().map(|report| report.identity_threshold),
                "retained_read_policy": governed
                    .as_ref()
                    .map(|report| report.retained_read_policy.clone()),
                "emit_removed_reads": governed.as_ref().map(|report| report.emit_removed_reads),
                "report_format": governed
                    .as_ref()
                    .map(|report| report.report_format.clone()),
                "retain_unmapped_pairs": governed.as_ref().map(|report| report.retain_unmapped_pairs),
                "reads_removed": governed.as_ref().map(|report| report.reads_removed),
                "bases_removed": governed.as_ref().map(|report| report.bases_removed),
                "host_fraction_removed": governed.as_ref().map(|report| report.host_fraction_removed),
                "removed_host_r1": governed.as_ref().map(|report| report.removed_host_r1.clone()),
                "removed_host_r2": governed.as_ref().and_then(|report| report.removed_host_r2.clone()),
                "raw_backend_report": governed.as_ref().and_then(|report| report.raw_backend_report.clone()),
                "raw_backend_report_format": governed.as_ref().and_then(|report| report.raw_backend_report_format.clone()),
                "report_json": report_path,
            }))
        }
        "fastq.normalize_primers" => {
            let report_path = stage_root.join("normalize_primers_report.json");
            let governed = std::fs::read_to_string(&report_path).ok().and_then(|raw| {
                bijux_dna_domain_fastq::observer::parse_normalize_primers_report(&raw).ok()
            });
            Some(serde_json::json!({
                "schema_version": "bijux.fastq.normalize_primers.extra_artifacts.v2",
                "stage": stage_id,
                "primer_set_id": governed.as_ref().map(|report| report.primer_set_id.clone()),
                "marker_id": governed.as_ref().and_then(|report| report.marker_id.clone()),
                "orientation_policy": governed.as_ref().map(|report| report.orientation_policy.clone()),
                "primer_orientation_report": governed.as_ref().map(|report| report.primer_orientation_report.clone()),
                "primer_stats_json": governed.as_ref().map(|report| report.primer_stats_json.clone()),
                "raw_backend_report_format": governed.as_ref().and_then(|report| report.raw_backend_report_format.clone()),
                "report_json": report_path,
            }))
        }
        "fastq.trim_terminal_damage" => {
            let report_path = execution
                .outputs
                .iter()
                .find(|path| {
                    path.file_name().and_then(|name| name.to_str())
                        == Some("trim_terminal_damage_report.json")
                })
                .cloned()
                .unwrap_or_else(|| stage_root.join("trim_terminal_damage_report.json"));
            let governed = std::fs::read_to_string(&report_path).ok().and_then(|raw| {
                bijux_dna_domain_fastq::observer::parse_terminal_damage_report(&raw).ok()
            });
            Some(serde_json::json!({
                "schema_version": "bijux.fastq.trim_terminal_damage.extra_artifacts.v2",
                "stage": stage_id,
                "tool": governed.as_ref().map(|report| report.tool_id.clone()),
                "paired_mode": governed.as_ref().map(|report| report.paired_mode),
                "threads": governed.as_ref().map(|report| report.threads),
                "reads_retained": governed.as_ref().and_then(|report| report.reads_out),
                "bases_removed": governed.as_ref().and_then(|report| {
                    report.bases_in.zip(report.bases_out).map(|(bases_in, bases_out)| {
                        bases_in.saturating_sub(bases_out)
                    })
                }),
                "damage_mode": governed.as_ref().map(|report| report.damage_mode),
                "execution_policy": governed.as_ref().map(|report| report.execution_policy),
                "trim_5p_bases": governed.as_ref().map(|report| report.trim_5p_bases),
                "trim_3p_bases": governed.as_ref().map(|report| report.trim_3p_bases),
                "requested_trim_5p_bases": governed.as_ref().and_then(|report| report.requested_trim_5p_bases),
                "requested_trim_3p_bases": governed.as_ref().and_then(|report| report.requested_trim_3p_bases),
                "udg_classification": governed.as_ref().map(|report| report.udg_classification.clone()),
                "ct_ga_asymmetry_pre_r1": governed.as_ref().and_then(|report| report.ct_ga_asymmetry_pre_r1),
                "ct_ga_asymmetry_post_r1": governed.as_ref().and_then(|report| report.ct_ga_asymmetry_post_r1),
                "ct_ga_asymmetry_pre_r2": governed.as_ref().and_then(|report| report.ct_ga_asymmetry_pre_r2),
                "ct_ga_asymmetry_post_r2": governed.as_ref().and_then(|report| report.ct_ga_asymmetry_post_r2),
                "used_fallback": governed.as_ref().map(|report| report.used_fallback),
                "backend_metrics": governed.as_ref().and_then(|report| report.backend_metrics.clone()),
                "raw_backend_report": governed.as_ref().and_then(|report| report.raw_backend_report.clone()),
                "raw_backend_report_format": governed.as_ref().and_then(|report| report.raw_backend_report_format.clone()),
                "report_json": report_path,
            }))
        }
        "fastq.remove_chimeras" => {
            let report_path = execution
                .outputs
                .iter()
                .find(|path| {
                    path.file_name().and_then(|name| name.to_str())
                        == Some("remove_chimeras_report.json")
                })
                .cloned()
                .unwrap_or_else(|| stage_root.join("remove_chimeras_report.json"));
            let governed = std::fs::read_to_string(&report_path).ok().and_then(|raw| {
                bijux_dna_domain_fastq::observer::parse_remove_chimeras_report(&raw).ok()
            });
            Some(serde_json::json!({
                "schema_version": "bijux.fastq.remove_chimeras.extra_artifacts.v2",
                "stage": stage_id,
                "method": governed.as_ref().map(|report| report.method.clone()),
                "detection_scope": governed.as_ref().map(|report| report.detection_scope.clone()),
                "used_fallback": governed.as_ref().map(|report| report.used_fallback),
                "chimeras_fasta": governed.as_ref().and_then(|report| report.chimeras_fasta.clone()),
                "uchime_report_tsv": governed.as_ref().and_then(|report| report.uchime_report_tsv.clone()),
                "raw_backend_report_format": governed.as_ref().and_then(|report| report.raw_backend_report_format.clone()),
                "report_json": report_path,
            }))
        }
        "fastq.cluster_otus" => {
            let report_path = execution
                .outputs
                .iter()
                .find(|path| {
                    path.file_name().and_then(|name| name.to_str())
                        == Some("cluster_otus_report.json")
                })
                .cloned()
                .unwrap_or_else(|| stage_root.join("cluster_otus_report.json"));
            let governed = std::fs::read_to_string(&report_path).ok().and_then(|raw| {
                bijux_dna_domain_fastq::observer::parse_cluster_otus_report(&raw).ok()
            });
            Some(serde_json::json!({
                "schema_version": "bijux.fastq.cluster_otus.extra_artifacts.v2",
                "stage": stage_id,
                "tool": governed.as_ref().map(|report| report.tool_id.clone()),
                "otu_identity": governed.as_ref().map(|report| report.otu_identity),
                "threads": governed.as_ref().map(|report| report.threads),
                "otu_table": governed.as_ref().map(|report| report.otu_table.clone()),
                "otu_representatives": governed.as_ref().map(|report| report.otu_representatives.clone()),
                "taxonomy_ready_fasta": governed.as_ref().map(|report| report.taxonomy_ready_fasta.clone()),
                "taxonomy_ready_fastq": governed.as_ref().map(|report| report.taxonomy_ready_fastq.clone()),
                "output_table_kind": governed.as_ref().map(|report| report.output_table_kind.clone()),
                "used_fallback": governed.as_ref().map(|report| report.used_fallback),
                "raw_backend_report": governed.as_ref().and_then(|report| report.raw_backend_report.clone()),
                "raw_backend_report_format": governed.as_ref().and_then(|report| report.raw_backend_report_format.clone()),
                "report_json": report_path,
            }))
        }
        "fastq.infer_asvs" => {
            let report_path = stage_root.join("infer_asvs_report.json");
            let governed = std::fs::read_to_string(&report_path).ok().and_then(|raw| {
                bijux_dna_domain_fastq::observer::parse_infer_asvs_report(&raw).ok()
            });
            Some(serde_json::json!({
                "schema_version": "bijux.fastq.infer_asvs.extra_artifacts.v2",
                "stage": stage_id,
                "tool": governed.as_ref().map(|report| report.tool_id.clone()),
                "paired_mode": governed.as_ref().map(|report| report.paired_mode),
                "denoising_method": governed.as_ref().map(|report| report.denoising_method.clone()),
                "pooling_mode": governed.as_ref().map(|report| report.pooling_mode.clone()),
                "chimera_policy": governed.as_ref().map(|report| report.chimera_policy.clone()),
                "asv_table_tsv": governed.as_ref().map(|report| report.asv_table_tsv.clone()),
                "asv_sequences_fasta": governed.as_ref().map(|report| report.asv_sequences_fasta.clone()),
                "taxonomy_ready_fasta": governed.as_ref().map(|report| report.taxonomy_ready_fasta.clone()),
                "taxonomy_ready_fastq": governed.as_ref().map(|report| report.taxonomy_ready_fastq.clone()),
                "representative_sequence_count": governed.as_ref().map(|report| report.representative_sequence_count),
                "used_fallback": governed.as_ref().map(|report| report.used_fallback),
                "report_json": report_path,
            }))
        }
        "fastq.normalize_abundance" => {
            let report_path = stage_root.join("normalize_abundance_report.json");
            let governed = std::fs::read_to_string(&report_path).ok().and_then(|raw| {
                bijux_dna_domain_fastq::observer::parse_normalize_abundance_report(&raw).ok()
            });
            Some(serde_json::json!({
                "schema_version": "bijux.fastq.normalize_abundance.extra_artifacts.v2",
                "stage": stage_id,
                "tool": governed.as_ref().map(|report| report.tool_id.clone()),
                "method": governed.as_ref().map(|report| report.method.clone()),
                "normalized_value_column": governed.as_ref().map(|report| report.normalized_value_column.clone()),
                "compositional_rule": governed.as_ref().map(|report| report.compositional_rule.clone()),
                "scale_factor": governed.as_ref().and_then(|report| report.scale_factor),
                "feature_count": governed.as_ref().map(|report| report.feature_count),
                "per_sample_sum_count": governed.as_ref().map(|report| report.per_sample_sums.len()),
                "report_json": report_path,
            }))
        }
        _ => None,
    };
    if let Some(v) = payload {
        bijux_dna_infra::atomic_write_json(&stage_root.join("stage.extra.json"), &v)
            .context("write stage.extra.json")?;
    }
    Ok(())
}

pub(super) fn write_stage_standardized_metrics(
    stage_root: &std::path::Path,
    stage_id: &str,
    out_dir: &std::path::Path,
    execution: &StageResultV1,
) -> Result<()> {
    standardized_metrics::write_stage_standardized_metrics(stage_root, stage_id, out_dir, execution)
}

#[cfg(test)]
mod stage_artifact_tests {
    use anyhow::Result;
    use bijux_dna_runner::step_runner::StageResultV1;

    use super::{emit_fastq_stage_extra_artifacts, write_stage_standardized_metrics};

    fn host_execution(stage_root: &std::path::Path) -> StageResultV1 {
        StageResultV1 {
            run_id: "host-fixture".to_string(),
            exit_code: 0,
            runtime_s: 5.0,
            memory_mb: 64.0,
            outputs: vec![stage_root.join("host_depletion_report.json")],
            metrics_path: None,
            stdout: String::new(),
            stderr: String::new(),
            command: "bowtie2".to_string(),
        }
    }

    fn write_host_report(stage_root: &std::path::Path) -> Result<()> {
        bijux_dna_infra::write_bytes(
            stage_root.join("host_depletion_report.json"),
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
        )?;
        Ok(())
    }

    fn detect_duplicates_premerge_execution(stage_root: &std::path::Path) -> StageResultV1 {
        StageResultV1 {
            run_id: "detect-duplicates-premerge-fixture".to_string(),
            exit_code: 0,
            runtime_s: 1.4,
            memory_mb: 12.0,
            outputs: vec![stage_root.join("duplicate_signal_report.json")],
            metrics_path: None,
            stdout: String::new(),
            stderr: String::new(),
            command: "bijux-dna".to_string(),
        }
    }

    fn write_detect_duplicates_premerge_report(stage_root: &std::path::Path) -> Result<()> {
        bijux_dna_infra::write_bytes(
            stage_root.join("duplicate_signal_report.json"),
            r#"{
                "schema_version": "bijux.fastq.detect_duplicates_premerge.report.v1",
                "stage": "fastq.detect_duplicates_premerge",
                "stage_id": "fastq.detect_duplicates_premerge",
                "tool_id": "bijux",
                "paired_mode": "paired_end",
                "duplicate_detection_policy": "report_only",
                "measurement_scope": "premerge_sequence_signature",
                "modifies_reads": false,
                "advisory_only": true,
                "reads_in": 12,
                "duplicate_signal_reads": 4,
                "duplicate_signal_fraction": 0.3333333333333333,
                "compared_read_pairs": 6
            }"#,
        )?;
        Ok(())
    }

    fn estimate_library_complexity_prealign_execution(
        stage_root: &std::path::Path,
    ) -> StageResultV1 {
        StageResultV1 {
            run_id: "estimate-library-complexity-prealign-fixture".to_string(),
            exit_code: 0,
            runtime_s: 0.8,
            memory_mb: 8.0,
            outputs: vec![stage_root.join("library_complexity_report.json")],
            metrics_path: None,
            stdout: String::new(),
            stderr: String::new(),
            command: "bijux-dna".to_string(),
        }
    }

    fn write_estimate_library_complexity_prealign_report(
        stage_root: &std::path::Path,
    ) -> Result<()> {
        bijux_dna_infra::write_bytes(
            stage_root.join("library_complexity_report.json"),
            r#"{
                "schema_version": "bijux.fastq.estimate_library_complexity_prealign.report.v1",
                "stage": "fastq.estimate_library_complexity_prealign",
                "stage_id": "fastq.estimate_library_complexity_prealign",
                "tool_id": "bijux",
                "paired_mode": "single_end",
                "complexity_policy": "prealign_kmer",
                "estimate_method": "kmer_redundancy",
                "modifies_reads": false,
                "advisory_only": true,
                "reads_in": 0,
                "estimated_unique_fraction": 0.0,
                "estimated_duplicate_fraction": 0.0,
                "insufficient_data_reason": "insufficient_reads_for_prealign_complexity_estimation",
                "kmer_size": 31
            }"#,
        )?;
        Ok(())
    }

    fn correct_errors_execution(stage_root: &std::path::Path) -> StageResultV1 {
        StageResultV1 {
            run_id: "correct-errors-fixture".to_string(),
            exit_code: 0,
            runtime_s: 2.2,
            memory_mb: 96.0,
            outputs: vec![stage_root.join("correct_report.json")],
            metrics_path: None,
            stdout: String::new(),
            stderr: String::new(),
            command: "rcorrector".to_string(),
        }
    }

    fn write_correct_errors_report(stage_root: &std::path::Path) -> Result<()> {
        bijux_dna_infra::write_bytes(
            stage_root.join("correct_report.json"),
            r#"{
                "schema_version": "bijux.fastq.correct_errors.report.v2",
                "stage": "fastq.correct_errors",
                "stage_id": "fastq.correct_errors",
                "tool_id": "rcorrector",
                "paired_mode": "paired_end",
                "threads": 4,
                "correction_engine": "rcorrector",
                "quality_encoding": "phred33",
                "kmer_size": null,
                "musket_kmer_budget": null,
                "genome_size": null,
                "max_memory_gb": 16,
                "trusted_kmer_artifact": null,
                "conservative_mode": false,
                "input_r1": "reads_R1.fastq.gz",
                "input_r2": "reads_R2.fastq.gz",
                "output_r1": "corrected_R1.fastq.gz",
                "output_r2": "corrected_R2.fastq.gz",
                "report_json": "correct_report.json",
                "corrected_reads": 200,
                "changed_reads": 18,
                "unchanged_reads": 182,
                "reads_in": 200,
                "reads_out": 200,
                "bases_in": 20000,
                "bases_out": 19950,
                "pairs_in": 100,
                "pairs_out": 100,
                "mean_q_before": 28.0,
                "mean_q_after": 29.1,
                "kmer_fix_rate": 0.05,
                "correction_effect": {
                    "outputs_changed": true,
                    "reads_delta": 0,
                    "bases_delta": -50,
                    "mean_q_delta": 1.1
                },
                "runtime_s": 2.2,
                "memory_mb": 96.0,
                "exit_code": 0,
                "raw_backend_report": "rcorrector.log",
                "raw_backend_report_format": "rcorrector_log",
                "backend_metrics": {
                    "trusted_kmers_loaded": false
                }
            }"#,
        )?;
        Ok(())
    }

    fn trim_terminal_damage_execution(stage_root: &std::path::Path) -> StageResultV1 {
        StageResultV1 {
            run_id: "trim-terminal-damage-fixture".to_string(),
            exit_code: 0,
            runtime_s: 4.0,
            memory_mb: 32.0,
            outputs: vec![stage_root.join("trim_terminal_damage_report.json")],
            metrics_path: None,
            stdout: String::new(),
            stderr: String::new(),
            command: bijux_dna_core::id_catalog::TOOL_CUTADAPT.to_string(),
        }
    }

    fn write_trim_terminal_damage_report(stage_root: &std::path::Path) -> Result<()> {
        bijux_dna_infra::write_bytes(
            stage_root.join("trim_terminal_damage_report.json"),
            format!(
                r#"{{
                "schema_version": "bijux.fastq.trim_terminal_damage.report.v2",
                "stage": "fastq.trim_terminal_damage",
                "stage_id": "fastq.trim_terminal_damage",
                "tool_id": "{tool_id}",
                "paired_mode": "single_end",
                "threads": 4,
                "damage_mode": "ancient",
                "execution_policy": "explicit_terminal_trim",
                "trim_5p_bases": 2,
                "trim_3p_bases": 1,
                "requested_trim_5p_bases": 2,
                "requested_trim_3p_bases": 1,
                "udg_classification": "non_udg",
                "input_r1": "reads.fastq.gz",
                "input_r2": null,
                "output_r1": "trimmed.fastq.gz",
                "output_r2": null,
                "reads_in": 100,
                "reads_out": 100,
                "bases_in": 1000,
                "bases_out": 700,
                "mean_q_before": 27.5,
                "mean_q_after": 28.0,
                "ct_ga_asymmetry_pre": 0.42,
                "ct_ga_asymmetry_post": 0.11,
                "ct_ga_asymmetry_pre_r1": 0.42,
                "ct_ga_asymmetry_post_r1": 0.11,
                "ct_ga_asymmetry_pre_r2": null,
                "ct_ga_asymmetry_post_r2": null,
                "terminal_base_composition_pre_r1": {{"C": 12}},
                "terminal_base_composition_post_r1": {{"C": 4}},
                "terminal_base_composition_pre_r2": null,
                "terminal_base_composition_post_r2": null,
                "raw_backend_report": "cutadapt.raw.json",
                "raw_backend_report_format": "cutadapt_json",
                "runtime_s": 4.0,
                "memory_mb": 32.0,
                "used_fallback": false,
                "backend_metrics": {{"reads_profiled_r1": 100}}
            }}"#,
                tool_id = bijux_dna_core::id_catalog::TOOL_CUTADAPT
            ),
        )?;
        Ok(())
    }

    fn trim_reads_execution(stage_root: &std::path::Path) -> StageResultV1 {
        StageResultV1 {
            run_id: "trim-reads-fixture".to_string(),
            exit_code: 0,
            runtime_s: 3.2,
            memory_mb: 40.0,
            outputs: vec![stage_root.join("trim_report.json")],
            metrics_path: None,
            stdout: String::new(),
            stderr: String::new(),
            command: "fastp".to_string(),
        }
    }

    fn write_trim_reads_report(stage_root: &std::path::Path) -> Result<()> {
        bijux_dna_infra::write_bytes(
            stage_root.join("trim_report.json"),
            r#"{
                "schema_version": "bijux.fastq.trim_reads.report.v2",
                "stage": "fastq.trim_reads",
                "stage_id": "fastq.trim_reads",
                "tool_id": "fastp",
                "paired_mode": "paired_end",
                "threads": 4,
                "trimming_backend": "fastp",
                "backend_mode": "enforced",
                "input_r1": "reads_R1.fastq.gz",
                "input_r2": "reads_R2.fastq.gz",
                "output_r1": "trimmed_R1.fastq.gz",
                "output_r2": "trimmed_R2.fastq.gz",
                "min_length": 30,
                "quality_cutoff": 20,
                "adapter_policy": "bank",
                "polyx_policy": "trim",
                "n_policy": "retain",
                "contaminant_policy": "none",
                "adapter_bank_id": "illumina",
                "adapter_bank_hash": "sha256:adapter",
                "adapter_preset": "illumina-default",
                "detected_adapter_source": "governed_pattern_scan",
                "adapter_overrides": {
                    "enable": ["AGATCGGAAGAGC"]
                },
                "prepared_adapter_bank": null,
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
                "pairs_in": 50,
                "pairs_out": 46,
                "mean_q_before": 28.0,
                "mean_q_after": 30.0,
                "effective_trim_params": {
                    "adapter_policy": "bank",
                    "min_length": 30,
                    "quality_cutoff": 20
                },
                "runtime_s": 3.2,
                "memory_mb": 40.0,
                "raw_backend_report": "trim.fastp.json",
                "raw_backend_report_format": "fastp_json"
            }"#,
        )?;
        Ok(())
    }

    fn filter_reads_execution(stage_root: &std::path::Path) -> StageResultV1 {
        StageResultV1 {
            run_id: "filter-reads-fixture".to_string(),
            exit_code: 0,
            runtime_s: 1.6,
            memory_mb: 64.0,
            outputs: vec![stage_root.join("filter_report.json")],
            metrics_path: None,
            stdout: String::new(),
            stderr: String::new(),
            command: "fastp".to_string(),
        }
    }

    fn write_filter_reads_report(stage_root: &std::path::Path) -> Result<()> {
        bijux_dna_infra::write_bytes(
            stage_root.join("filter_report.json"),
            r#"{
                "schema_version": "bijux.fastq.filter_reads.report.v3",
                "stage": "fastq.filter_reads",
                "stage_id": "fastq.filter_reads",
                "tool_id": "fastp",
                "paired_mode": "paired_end",
                "threads": 8,
                "input_r1": "reads_R1.fastq.gz",
                "input_r2": "reads_R2.fastq.gz",
                "output_r1": "filtered_R1.fastq.gz",
                "output_r2": "filtered_R2.fastq.gz",
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
                "pairs_in": 50,
                "pairs_out": 47,
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
        )?;
        Ok(())
    }

    fn trim_polyg_execution(stage_root: &std::path::Path) -> StageResultV1 {
        StageResultV1 {
            run_id: "trim-polyg-fixture".to_string(),
            exit_code: 0,
            runtime_s: 3.5,
            memory_mb: 28.0,
            outputs: vec![stage_root.join("trim_polyg_tails_report.json")],
            metrics_path: None,
            stdout: String::new(),
            stderr: String::new(),
            command: bijux_dna_core::id_catalog::TOOL_FASTP.to_string(),
        }
    }

    fn write_trim_polyg_report(stage_root: &std::path::Path) -> Result<()> {
        bijux_dna_infra::write_bytes(
            stage_root.join("trim_polyg_tails_report.json"),
            format!(
                r#"{{
                "schema_version": "bijux.fastq.trim_polyg_tails.report.v2",
                "stage": "fastq.trim_polyg_tails",
                "stage_id": "fastq.trim_polyg_tails",
                "tool_id": "{tool_id}",
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
                "trimmed_tail_count": 4,
                "bases_trimmed_polyg": 90,
                "polyx_bank_id": "polyx",
                "polyx_bank_hash": "sha256:polyx",
                "polyx_preset": "illumina_twocolor",
                "runtime_s": 3.5,
                "memory_mb": 28.0,
                "raw_backend_report": "trim_polyg_tails_report.fastp.json",
                "raw_backend_report_format": "fastp_json",
                "backend_metrics": {{
                    "schema_version": "bijux.fastp.metrics.v1",
                    "passed_filter_reads": 96
                }}
            }}"#,
                tool_id = bijux_dna_core::id_catalog::TOOL_FASTP
            ),
        )?;
        Ok(())
    }

    fn filter_low_complexity_execution(stage_root: &std::path::Path) -> StageResultV1 {
        StageResultV1 {
            run_id: "filter-low-complexity-fixture".to_string(),
            exit_code: 0,
            runtime_s: 1.1,
            memory_mb: 64.0,
            outputs: vec![stage_root.join("low_complexity_report.json")],
            metrics_path: None,
            stdout: String::new(),
            stderr: String::new(),
            command: "bbduk".to_string(),
        }
    }

    fn write_filter_low_complexity_report(stage_root: &std::path::Path) -> Result<()> {
        bijux_dna_infra::write_bytes(
            stage_root.join("low_complexity_report.json"),
            r#"{
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
            }"#,
        )?;
        Ok(())
    }

    fn remove_duplicates_execution(stage_root: &std::path::Path) -> StageResultV1 {
        StageResultV1 {
            run_id: "remove-duplicates-fixture".to_string(),
            exit_code: 0,
            runtime_s: 2.7,
            memory_mb: 48.0,
            outputs: vec![stage_root.join("deduplicate_report.json")],
            metrics_path: None,
            stdout: String::new(),
            stderr: String::new(),
            command: "clumpify".to_string(),
        }
    }

    fn write_remove_duplicates_report(stage_root: &std::path::Path) -> Result<()> {
        bijux_dna_infra::write_bytes(
            stage_root.join("deduplicate_report.json"),
            r#"{
                "schema_version": "bijux.fastq.remove_duplicates.report.v2",
                "stage": "fastq.remove_duplicates",
                "stage_id": "fastq.remove_duplicates",
                "tool_id": "clumpify",
                "paired_mode": "paired_end",
                "threads": 6,
                "dedup_mode": "optical_aware",
                "keep_order": false,
                "input_r1": "reads_R1.fastq.gz",
                "input_r2": "reads_R2.fastq.gz",
                "output_r1": "dedup_R1.fastq.gz",
                "output_r2": "dedup_R2.fastq.gz",
                "reads_in": 200,
                "reads_out": 172,
                "reads_in_r2": 200,
                "reads_out_r2": 172,
                "pairs_in": 200,
                "pairs_out": 172,
                "pair_count_match": true,
                "duplicates_removed": 28,
                "dedup_rate": 0.14,
                "duplicate_classes_tsv": "duplicate_classes.tsv",
                "duplicate_provenance_json": "duplicate_provenance.json",
                "duplicate_classes": [
                    {"class": "duplicate", "reads_removed": 20, "paired_mode": "paired_end"},
                    {"class": "optical_duplicate", "reads_removed": 8, "paired_mode": "paired_end"}
                ],
                "raw_backend_report": "clumpify.log",
                "raw_backend_report_format": "clumpify_log",
                "runtime_s": 2.7,
                "memory_mb": 48.0
            }"#,
        )?;
        Ok(())
    }

    fn merge_execution(stage_root: &std::path::Path) -> StageResultV1 {
        StageResultV1 {
            run_id: "merge-fixture".to_string(),
            exit_code: 0,
            runtime_s: 2.3,
            memory_mb: 48.0,
            outputs: vec![stage_root.join("merge_report.json")],
            metrics_path: None,
            stdout: String::new(),
            stderr: String::new(),
            command: "pear".to_string(),
        }
    }

    fn write_merge_report(stage_root: &std::path::Path) -> Result<()> {
        bijux_dna_infra::write_bytes(
            stage_root.join("merge_report.json"),
            r#"{
                "schema_version": "bijux.fastq.merge_pairs.report.v2",
                "stage": "fastq.merge_pairs",
                "stage_id": "fastq.merge_pairs",
                "tool_id": "pear",
                "paired_mode": "paired_end",
                "merge_engine": "pear",
                "threads": 4,
                "merge_overlap": 22,
                "min_len": 120,
                "unmerged_read_policy": "emit_unmerged_pairs",
                "input_r1": "reads_R1.fastq.gz",
                "input_r2": "reads_R2.fastq.gz",
                "merged_reads": "pear.assembled.fastq",
                "unmerged_reads_r1": "pear.unassembled.forward.fastq",
                "unmerged_reads_r2": "pear.unassembled.reverse.fastq",
                "reads_r1": 100,
                "reads_r2": 100,
                "reads_merged": 88,
                "reads_unmerged": 12,
                "merge_rate": 0.88,
                "runtime_s": 2.3,
                "memory_mb": 48.0,
                "raw_backend_report": "pear.log",
                "raw_backend_report_format": "pear_log"
            }"#,
        )?;
        Ok(())
    }

    #[test]
    fn host_extra_artifacts_prefer_governed_report() -> Result<()> {
        let temp = tempfile::tempdir()?;
        write_host_report(temp.path())?;
        emit_fastq_stage_extra_artifacts(
            temp.path(),
            "fastq.deplete_host",
            &host_execution(temp.path()),
        )?;

        let extra: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(temp.path().join("stage.extra.json"))?)?;
        assert_eq!(extra["tool"], serde_json::json!("bowtie2"));
        assert_eq!(extra["reference_catalog_id"], serde_json::json!("host_reference"));
        assert_eq!(extra["reads_removed"], serde_json::json!(30));
        assert_eq!(extra["host_fraction_removed"], serde_json::json!(0.30));
        Ok(())
    }

    #[test]
    fn merge_pairs_extra_artifacts_prefer_governed_report() -> Result<()> {
        let temp = tempfile::tempdir()?;
        write_merge_report(temp.path())?;
        emit_fastq_stage_extra_artifacts(
            temp.path(),
            "fastq.merge_pairs",
            &merge_execution(temp.path()),
        )?;

        let extra: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(temp.path().join("stage.extra.json"))?)?;
        assert_eq!(extra["tool"], serde_json::json!("pear"));
        assert_eq!(extra["threads"], serde_json::json!(4));
        assert_eq!(extra["reads_r1"], serde_json::json!(100));
        assert_eq!(extra["reads_r2"], serde_json::json!(100));
        assert_eq!(extra["input_pair_count"], serde_json::json!(100));
        assert_eq!(extra["reads_merged"], serde_json::json!(88));
        assert_eq!(extra["reads_unmerged"], serde_json::json!(12));
        assert_eq!(extra["merged_pair_count"], serde_json::json!(88));
        assert_eq!(extra["unmerged_pair_count"], serde_json::json!(12));
        assert_eq!(extra["discarded_pair_count"], serde_json::json!(0));
        assert_eq!(extra["merge_rate"], serde_json::json!(0.88));
        assert_eq!(extra["raw_backend_report"], serde_json::json!("pear.log"));
        assert_eq!(extra["raw_backend_report_format"], serde_json::json!("pear_log"));
        Ok(())
    }

    #[test]
    fn detect_duplicates_premerge_extra_artifacts_prefer_governed_report() -> Result<()> {
        let temp = tempfile::tempdir()?;
        write_detect_duplicates_premerge_report(temp.path())?;
        emit_fastq_stage_extra_artifacts(
            temp.path(),
            "fastq.detect_duplicates_premerge",
            &detect_duplicates_premerge_execution(temp.path()),
        )?;

        let extra: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(temp.path().join("stage.extra.json"))?)?;
        assert_eq!(extra["tool"], serde_json::json!("bijux"));
        assert_eq!(extra["paired_mode"], serde_json::json!("paired_end"));
        assert_eq!(extra["duplicate_detection_policy"], serde_json::json!("report_only"));
        assert_eq!(extra["measurement_scope"], serde_json::json!("premerge_sequence_signature"));
        assert_eq!(extra["reads_in"], serde_json::json!(12));
        assert_eq!(extra["duplicate_count"], serde_json::json!(4));
        assert_eq!(extra["duplicate_fraction"], serde_json::json!(0.3333333333333333));
        assert_eq!(extra["inspected_pair_count"], serde_json::json!(6));
        assert_eq!(
            extra["report_json"],
            serde_json::json!(temp.path().join("duplicate_signal_report.json"))
        );
        Ok(())
    }

    #[test]
    fn estimate_library_complexity_prealign_extra_artifacts_prefer_governed_report() -> Result<()> {
        let temp = tempfile::tempdir()?;
        write_estimate_library_complexity_prealign_report(temp.path())?;
        emit_fastq_stage_extra_artifacts(
            temp.path(),
            "fastq.estimate_library_complexity_prealign",
            &estimate_library_complexity_prealign_execution(temp.path()),
        )?;

        let extra: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(temp.path().join("stage.extra.json"))?)?;
        assert_eq!(extra["tool"], serde_json::json!("bijux"));
        assert_eq!(extra["paired_mode"], serde_json::json!("single_end"));
        assert_eq!(extra["complexity_policy"], serde_json::json!("prealign_kmer"));
        assert_eq!(extra["estimate_method"], serde_json::json!("kmer_redundancy"));
        assert_eq!(extra["reads_in"], serde_json::json!(0));
        assert_eq!(extra["estimated_complexity"], serde_json::Value::Null);
        assert_eq!(extra["estimated_duplicate_fraction"], serde_json::json!(0.0));
        assert_eq!(
            extra["insufficient_data_reason"],
            serde_json::json!("insufficient_reads_for_prealign_complexity_estimation")
        );
        assert_eq!(extra["complexity_status"], serde_json::json!("insufficient_data"));
        assert_eq!(
            extra["report_json"],
            serde_json::json!(temp.path().join("library_complexity_report.json"))
        );
        Ok(())
    }

    #[test]
    fn correct_errors_extra_artifacts_prefer_governed_report() -> Result<()> {
        let temp = tempfile::tempdir()?;
        write_correct_errors_report(temp.path())?;
        emit_fastq_stage_extra_artifacts(
            temp.path(),
            "fastq.correct_errors",
            &correct_errors_execution(temp.path()),
        )?;

        let extra: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(temp.path().join("stage.extra.json"))?)?;
        assert_eq!(extra["tool"], serde_json::json!("rcorrector"));
        assert_eq!(extra["threads"], serde_json::json!(4));
        assert_eq!(extra["corrected_reads_r1"], serde_json::json!("corrected_R1.fastq.gz"));
        assert_eq!(extra["corrected_reads_r2"], serde_json::json!("corrected_R2.fastq.gz"));
        assert_eq!(extra["corrected_reads"], serde_json::json!(200));
        assert_eq!(extra["changed_reads"], serde_json::json!(18));
        assert_eq!(extra["unchanged_reads"], serde_json::json!(182));
        assert_eq!(extra["raw_backend_report"], serde_json::json!("rcorrector.log"));
        assert_eq!(
            extra["report_json"],
            serde_json::json!(temp.path().join("correct_report.json"))
        );
        Ok(())
    }

    #[test]
    fn host_standardized_metrics_writer_uses_governed_report() -> Result<()> {
        let temp = tempfile::tempdir()?;
        write_host_report(temp.path())?;
        write_stage_standardized_metrics(
            temp.path(),
            "fastq.deplete_host",
            temp.path(),
            &host_execution(temp.path()),
        )?;

        let metrics: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(
            temp.path().join("stage.metrics.standardized.json"),
        )?)?;
        assert_eq!(metrics["tool"], serde_json::json!("bowtie2"));
        assert_eq!(metrics["reads_removed"], serde_json::json!(30));
        assert_eq!(metrics["host_fraction_removed"], serde_json::json!(0.30));
        Ok(())
    }

    #[test]
    fn detect_duplicates_premerge_standardized_metrics_writer_uses_governed_report() -> Result<()> {
        let temp = tempfile::tempdir()?;
        write_detect_duplicates_premerge_report(temp.path())?;
        write_stage_standardized_metrics(
            temp.path(),
            "fastq.detect_duplicates_premerge",
            temp.path(),
            &detect_duplicates_premerge_execution(temp.path()),
        )?;

        let metrics: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(
            temp.path().join("stage.metrics.standardized.json"),
        )?)?;
        assert_eq!(metrics["tool"], serde_json::json!("bijux"));
        assert_eq!(metrics["reads_in"], serde_json::json!(12));
        assert_eq!(metrics["duplicate_count"], serde_json::json!(4));
        assert_eq!(metrics["duplicate_fraction"], serde_json::json!(0.3333333333333333));
        assert_eq!(metrics["inspected_pair_count"], serde_json::json!(6));
        Ok(())
    }

    #[test]
    fn estimate_library_complexity_prealign_standardized_metrics_writer_uses_governed_report(
    ) -> Result<()> {
        let temp = tempfile::tempdir()?;
        write_estimate_library_complexity_prealign_report(temp.path())?;
        write_stage_standardized_metrics(
            temp.path(),
            "fastq.estimate_library_complexity_prealign",
            temp.path(),
            &estimate_library_complexity_prealign_execution(temp.path()),
        )?;

        let metrics: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(
            temp.path().join("stage.metrics.standardized.json"),
        )?)?;
        assert_eq!(metrics["tool"], serde_json::json!("bijux"));
        assert_eq!(metrics["reads_in"], serde_json::json!(0));
        assert_eq!(metrics["estimated_complexity"], serde_json::Value::Null);
        assert_eq!(metrics["estimated_duplicate_fraction"], serde_json::json!(0.0));
        assert_eq!(
            metrics["insufficient_data_reason"],
            serde_json::json!("insufficient_reads_for_prealign_complexity_estimation")
        );
        assert_eq!(metrics["complexity_status"], serde_json::json!("insufficient_data"));
        Ok(())
    }

    #[test]
    fn correct_errors_standardized_metrics_writer_uses_governed_report() -> Result<()> {
        let temp = tempfile::tempdir()?;
        write_correct_errors_report(temp.path())?;
        write_stage_standardized_metrics(
            temp.path(),
            "fastq.correct_errors",
            temp.path(),
            &correct_errors_execution(temp.path()),
        )?;

        let metrics: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(
            temp.path().join("stage.metrics.standardized.json"),
        )?)?;
        assert_eq!(metrics["tool"], serde_json::json!("rcorrector"));
        assert_eq!(metrics["corrected_reads_r1"], serde_json::json!("corrected_R1.fastq.gz"));
        assert_eq!(metrics["corrected_reads_r2"], serde_json::json!("corrected_R2.fastq.gz"));
        assert_eq!(metrics["corrected_reads"], serde_json::json!(200));
        assert_eq!(metrics["changed_reads"], serde_json::json!(18));
        assert_eq!(metrics["unchanged_reads"], serde_json::json!(182));
        Ok(())
    }

    #[test]
    fn trim_terminal_damage_extra_artifacts_prefer_governed_report() -> Result<()> {
        let temp = tempfile::tempdir()?;
        write_trim_terminal_damage_report(temp.path())?;
        emit_fastq_stage_extra_artifacts(
            temp.path(),
            "fastq.trim_terminal_damage",
            &trim_terminal_damage_execution(temp.path()),
        )?;

        let extra: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(temp.path().join("stage.extra.json"))?)?;
        assert_eq!(extra["tool"], serde_json::json!(bijux_dna_core::id_catalog::TOOL_CUTADAPT));
        assert_eq!(extra["threads"], serde_json::json!(4));
        assert_eq!(extra["trim_5p_bases"], serde_json::json!(2));
        assert_eq!(extra["trim_3p_bases"], serde_json::json!(1));
        assert_eq!(extra["reads_retained"], serde_json::json!(100));
        assert_eq!(extra["bases_removed"], serde_json::json!(300));
        assert_eq!(extra["raw_backend_report"], serde_json::json!("cutadapt.raw.json"));
        assert_eq!(extra["used_fallback"], serde_json::json!(false));
        Ok(())
    }

    #[test]
    fn trim_reads_extra_artifacts_prefer_governed_report() -> Result<()> {
        let temp = tempfile::tempdir()?;
        write_trim_reads_report(temp.path())?;
        emit_fastq_stage_extra_artifacts(
            temp.path(),
            "fastq.trim_reads",
            &trim_reads_execution(temp.path()),
        )?;

        let extra: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(temp.path().join("stage.extra.json"))?)?;
        assert_eq!(extra["tool"], serde_json::json!("fastp"));
        assert_eq!(extra["trimmed_reads_r1"], serde_json::json!("trimmed_R1.fastq.gz"));
        assert_eq!(extra["trimmed_reads_r2"], serde_json::json!("trimmed_R2.fastq.gz"));
        assert_eq!(extra["report_json"], serde_json::json!(temp.path().join("trim_report.json")));
        assert_eq!(extra["reads_retained"], serde_json::json!(92));
        assert_eq!(extra["reads_dropped"], serde_json::json!(8));
        assert_eq!(extra["bases_removed"], serde_json::json!(150));
        assert_eq!(extra["raw_backend_report"], serde_json::json!("trim.fastp.json"));
        Ok(())
    }

    #[test]
    fn filter_reads_extra_artifacts_prefer_governed_report() -> Result<()> {
        let temp = tempfile::tempdir()?;
        write_filter_reads_report(temp.path())?;
        emit_fastq_stage_extra_artifacts(
            temp.path(),
            "fastq.filter_reads",
            &filter_reads_execution(temp.path()),
        )?;

        let extra: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(temp.path().join("stage.extra.json"))?)?;
        assert_eq!(extra["tool"], serde_json::json!("fastp"));
        assert_eq!(extra["filtered_reads_r1"], serde_json::json!("filtered_R1.fastq.gz"));
        assert_eq!(extra["filtered_reads_r2"], serde_json::json!("filtered_R2.fastq.gz"));
        assert_eq!(extra["report_json"], serde_json::json!(temp.path().join("filter_report.json")));
        assert_eq!(extra["reads_retained"], serde_json::json!(95));
        assert_eq!(extra["reads_removed"], serde_json::json!(5));
        assert_eq!(extra["reads_removed_by_n"], serde_json::json!(2));
        assert_eq!(extra["reads_removed_by_length"], serde_json::json!(1));
        assert_eq!(extra["raw_backend_report"], serde_json::json!("fastp.filter.json"));
        Ok(())
    }

    #[test]
    fn filter_low_complexity_extra_artifacts_prefer_governed_report() -> Result<()> {
        let temp = tempfile::tempdir()?;
        write_filter_low_complexity_report(temp.path())?;
        emit_fastq_stage_extra_artifacts(
            temp.path(),
            "fastq.filter_low_complexity",
            &filter_low_complexity_execution(temp.path()),
        )?;

        let extra: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(temp.path().join("stage.extra.json"))?)?;
        assert_eq!(extra["tool"], serde_json::json!("bbduk"));
        assert_eq!(extra["filtered_fastq_r1"], serde_json::json!("filtered.fastq.gz"));
        assert_eq!(extra["filtered_fastq_r2"], serde_json::Value::Null);
        assert_eq!(extra["reads_removed_low_complexity"], serde_json::json!(8));
        assert_eq!(extra["polyx_threshold"], serde_json::json!(20));
        assert_eq!(extra["raw_backend_report"], serde_json::json!("bbduk.low_complexity.stats"));
        Ok(())
    }

    #[test]
    fn trim_polyg_extra_artifacts_prefer_governed_report() -> Result<()> {
        let temp = tempfile::tempdir()?;
        write_trim_polyg_report(temp.path())?;
        emit_fastq_stage_extra_artifacts(
            temp.path(),
            "fastq.trim_polyg_tails",
            &trim_polyg_execution(temp.path()),
        )?;

        let extra: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(temp.path().join("stage.extra.json"))?)?;
        assert_eq!(extra["tool"], serde_json::json!(bijux_dna_core::id_catalog::TOOL_FASTP));
        assert_eq!(extra["threads"], serde_json::json!(6));
        assert_eq!(extra["trim_polyg"], serde_json::json!(true));
        assert_eq!(extra["trimmed_tail_count"], serde_json::json!(4));
        assert_eq!(extra["bases_trimmed_polyg"], serde_json::json!(90));
        assert_eq!(extra["polyx_bank_id"], serde_json::json!("polyx"));
        assert_eq!(extra["polyx_preset"], serde_json::json!("illumina_twocolor"));
        assert_eq!(
            extra["raw_backend_report"],
            serde_json::json!("trim_polyg_tails_report.fastp.json")
        );
        assert_eq!(
            extra["report_json"],
            serde_json::json!(temp.path().join("trim_polyg_tails_report.json"))
        );
        Ok(())
    }

    #[test]
    fn remove_duplicates_extra_artifacts_preserve_governed_dedup_contract() -> Result<()> {
        let temp = tempfile::tempdir()?;
        write_remove_duplicates_report(temp.path())?;
        emit_fastq_stage_extra_artifacts(
            temp.path(),
            "fastq.remove_duplicates",
            &remove_duplicates_execution(temp.path()),
        )?;

        let extra: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(temp.path().join("stage.extra.json"))?)?;
        assert_eq!(extra["tool"], serde_json::json!("clumpify"));
        assert_eq!(extra["threads"], serde_json::json!(6));
        assert_eq!(extra["dedup_mode"], serde_json::json!("optical_aware"));
        assert_eq!(extra["keep_order"], serde_json::json!(false));
        assert_eq!(extra["reads_in"], serde_json::json!(200));
        assert_eq!(extra["reads_out"], serde_json::json!(172));
        assert_eq!(extra["input_reads"], serde_json::json!(200));
        assert_eq!(extra["duplicate_reads"], serde_json::json!(28));
        assert_eq!(extra["unique_reads"], serde_json::json!(172));
        assert_eq!(extra["output_reads"], serde_json::json!(172));
        assert_eq!(extra["pairs_in"], serde_json::json!(200));
        assert_eq!(extra["pairs_out"], serde_json::json!(172));
        assert_eq!(extra["duplicates_removed"], serde_json::json!(28));
        assert_eq!(extra["dedup_rate"], serde_json::json!(0.14));
        assert_eq!(extra["pair_count_match"], serde_json::json!(true));
        assert_eq!(extra["raw_backend_report"], serde_json::json!("clumpify.log"));
        Ok(())
    }

    #[test]
    fn extract_umis_extra_artifacts_preserve_governed_umi_contract() -> Result<()> {
        let temp = tempfile::tempdir()?;
        bijux_dna_infra::write_bytes(
            temp.path().join("umi_report.json"),
            r#"{
                "schema_version": "bijux.fastq.extract_umis.report.v2",
                "stage": "fastq.extract_umis",
                "stage_id": "fastq.extract_umis",
                "tool_id": "umi_tools",
                "paired_mode": "paired_end",
                "threads": 2,
                "umi_pattern": "NNNNNNNN",
                "extraction_location": "read1_prefix",
                "read_name_transform": "append_to_header",
                "failed_extraction_policy": "refuse_stage",
                "grouping_policy": "pair_aware",
                "downstream_dedup_policy": "sequence_identity_recommended",
                "downstream_propagation": "header_and_report",
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
                "reads_with_umi": 196,
                "failed_extractions": 4,
                "mean_q_before": 30.0,
                "mean_q_after": 30.0,
                "runtime_s": 1.4,
                "memory_mb": 64.0,
                "exit_code": 0,
                "raw_backend_report": "umi_tools.extract.log",
                "raw_backend_report_format": "umi_tools_log",
                "backend_metrics": {
                    "reads_with_umi_fraction": 0.98
                }
            }"#,
        )?;
        emit_fastq_stage_extra_artifacts(
            temp.path(),
            "fastq.extract_umis",
            &StageResultV1 {
                run_id: "extract-umis-fixture".to_string(),
                exit_code: 0,
                runtime_s: 1.4,
                memory_mb: 64.0,
                outputs: vec![temp.path().join("umi_report.json")],
                metrics_path: None,
                stdout: String::new(),
                stderr: String::new(),
                command: "umi_tools".to_string(),
            },
        )?;

        let extra: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(temp.path().join("stage.extra.json"))?)?;
        assert_eq!(extra["tool"], serde_json::json!("umi_tools"));
        assert_eq!(extra["umi_pattern"], serde_json::json!("NNNNNNNN"));
        assert_eq!(extra["tag_header_format"], serde_json::json!("append_to_header"));
        assert_eq!(extra["downstream_propagation"], serde_json::json!("header_and_report"));
        assert_eq!(extra["umi_reads_r1"], serde_json::json!("umi_reads_R1.fastq.gz"));
        assert_eq!(extra["umi_reads_r2"], serde_json::json!("umi_reads_R2.fastq.gz"));
        assert_eq!(extra["reads_with_umi"], serde_json::json!(196));
        assert_eq!(extra["failed_extractions"], serde_json::json!(4));
        assert_eq!(extra["extracted_umi_count"], serde_json::json!(196));
        assert_eq!(extra["invalid_umi_count"], serde_json::json!(4));
        assert_eq!(extra["raw_backend_report"], serde_json::json!("umi_tools.extract.log"));
        Ok(())
    }

    fn report_qc_execution(stage_root: &std::path::Path) -> StageResultV1 {
        StageResultV1 {
            run_id: "report-qc-fixture".to_string(),
            exit_code: 0,
            runtime_s: 2.0,
            memory_mb: 96.0,
            outputs: vec![stage_root.join("report_qc_report.json")],
            metrics_path: None,
            stdout: String::new(),
            stderr: String::new(),
            command: "multiqc".to_string(),
        }
    }

    fn write_report_qc_report(stage_root: &std::path::Path) -> Result<()> {
        bijux_dna_infra::write_bytes(
            stage_root.join("report_qc_report.json"),
            format!(
                r#"{{
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
                "pairs_in": null,
                "pairs_out": null,
                "mean_q": 31.0,
                "contamination_rate": 0.23,
                "adapter_content_max": 0.12,
                "adapter_content_mean": 0.04,
                "duplication_rate": 0.11,
                "n_rate": 0.002,
                "kmer_warning_count": 3,
                "overrepresented_sequence_count": 2,
                "multiqc_sample_count": 2,
                "multiqc_module_count": 5,
                "raw_fastqc_dir": "raw_fastqc",
                "trimmed_fastqc_dir": "fastqc_trimmed",
                "multiqc_report": "multiqc_report.html",
                "multiqc_data": "multiqc_data",
                "governed_qc_input_count": 2,
                "governed_qc_contributor_stage_ids": ["fastq.detect_adapters", "fastq.screen_taxonomy"],
                "governed_qc_contributor_tool_ids": [
                    "fastqc",
                    "{kraken2_tool}"
                ],
                "governed_qc_contributors": [
                    {{
                        "contributor_id": "fastq.detect_adapters.fastqc",
                        "stage_id": "fastq.detect_adapters",
                        "tool_id": "fastqc",
                        "artifact_id": "report_json",
                        "artifact_role": "report_json",
                        "path": "adapter_report.json"
                    }}
                ],
                "governed_qc_lineage_hash": "lineage",
                "governed_qc_inputs_manifest": "governed_qc_inputs_manifest.json",
                "runtime_s": 2.0,
                "memory_mb": 96.0,
                "exit_code": 0
            }}"#,
                kraken2_tool = bijux_dna_core::id_catalog::TOOL_KRAKEN2
            ),
        )?;
        Ok(())
    }

    #[test]
    fn report_qc_extra_artifacts_preserve_governed_summary_contract() -> Result<()> {
        let temp = tempfile::tempdir()?;
        write_report_qc_report(temp.path())?;
        emit_fastq_stage_extra_artifacts(
            temp.path(),
            "fastq.report_qc",
            &report_qc_execution(temp.path()),
        )?;

        let extra: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(temp.path().join("stage.extra.json"))?)?;
        assert_eq!(extra["aggregation_scope"], serde_json::json!("governed_qc_artifacts"));
        assert_eq!(extra["contamination_rate"], serde_json::json!(0.23));
        assert_eq!(extra["adapter_content_max"], serde_json::json!(0.12));
        assert_eq!(extra["governed_qc_contributors"][0]["tool_id"], serde_json::json!("fastqc"));
        assert_eq!(
            extra["governed_qc_inputs_manifest"],
            serde_json::json!("governed_qc_inputs_manifest.json")
        );
        Ok(())
    }
}
