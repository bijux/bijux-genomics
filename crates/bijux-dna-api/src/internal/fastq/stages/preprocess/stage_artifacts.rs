fn emit_fastq_stage_extra_artifacts(
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
                bijux_dna_stages_fastq::observer::parse_index_reference_report(&raw).ok()
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
                bijux_dna_stages_fastq::observer::parse_detect_adapters_report(&raw).ok()
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
                "candidate_adapter_count": governed.as_ref().map(|report| report.candidate_adapter_count),
                "adapter_trimmed_fraction": governed.as_ref().and_then(|report| report.adapter_trimmed_fraction),
                "adapter_evidence_dir": governed.as_ref().map(|report| report.adapter_evidence_dir.clone()),
                "report_json": report_path,
            }))
        }
        "fastq.filter_reads" => {
            let report_path = execution
                .outputs
                .iter()
                .find(|path| path.file_name().and_then(|name| name.to_str()) == Some("filter_report.json"))
                .cloned()
                .unwrap_or_else(|| stage_root.join("filter_report.json"));
            let governed = std::fs::read_to_string(&report_path).ok().and_then(|raw| {
                bijux_dna_stages_fastq::observer::parse_filter_reads_report(&raw).ok()
            });
            Some(serde_json::json!({
                "schema_version": "bijux.fastq.filter_reads.extra_artifacts.v2",
                "stage": stage_id,
                "tool": governed.as_ref().map(|report| report.tool_id.clone()),
                "paired_mode": governed.as_ref().map(|report| report.paired_mode),
                "threads": governed.as_ref().map(|report| report.threads),
                "max_n": governed.as_ref().and_then(|report| report.max_n),
                "max_n_fraction": governed.as_ref().and_then(|report| report.max_n_fraction),
                "max_n_count": governed.as_ref().and_then(|report| report.max_n_count),
                "low_complexity_threshold": governed.as_ref().and_then(|report| report.low_complexity_threshold),
                "entropy_threshold": governed.as_ref().and_then(|report| report.entropy_threshold),
                "n_policy": governed.as_ref().and_then(|report| report.n_policy.clone()),
                "polyx_policy": governed.as_ref().and_then(|report| report.polyx_policy.clone()),
                "contaminant_db": governed.as_ref().and_then(|report| report.contaminant_db.clone()),
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
        "fastq.filter_low_complexity" => {
            let report_path = stage_root.join("low_complexity_report.json");
            let governed = std::fs::read_to_string(&report_path).ok().and_then(|raw| {
                bijux_dna_stages_fastq::observer::parse_filter_low_complexity_report(&raw).ok()
            });
            Some(serde_json::json!({
                "schema_version": "bijux.fastq.filter_low_complexity.extra_artifacts.v2",
                "stage": stage_id,
                "tool": governed.as_ref().map(|report| report.tool_id.clone()),
                "paired_mode": governed.as_ref().map(|report| report.paired_mode),
                "threads": governed.as_ref().map(|report| report.threads),
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
                bijux_dna_stages_fastq::observer::parse_profile_reads_report(&raw).ok()
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
                bijux_dna_stages_fastq::observer::parse_profile_read_lengths_report(&raw).ok()
            });
            Some(serde_json::json!({
                "schema_version": "bijux.fastq.profile_read_lengths.extra_artifacts.v2",
                "stage": stage_id,
                "paired_mode": governed.as_ref().map(|report| report.paired_mode),
                "histogram_bins": governed.as_ref().map(|report| report.histogram_bins),
                "histogram_entry_count": governed.as_ref().map(|report| report.histogram.len()),
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
                bijux_dna_stages_fastq::observer::parse_profile_overrepresented_report(&raw).ok()
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
                bijux_dna_stages_fastq::observer::parse_trim_polyg_report(&raw).ok()
            });
            Some(serde_json::json!({
                "schema_version": "bijux.fastq.trim_polyg_tails.extra_artifacts.v2",
                "stage": stage_id,
                "trim_polyg": governed.as_ref().map(|report| report.trim_polyg),
                "min_polyg_run": governed.as_ref().map(|report| report.min_polyg_run),
                "paired_mode": governed.as_ref().map(|report| report.paired_mode),
                "bases_trimmed_polyg": governed.as_ref().and_then(|report| report.bases_trimmed_polyg),
                "polyx_bank_hash": governed.as_ref().and_then(|report| report.polyx_bank_hash.clone()),
                "raw_backend_report_format": governed.as_ref().and_then(|report| report.raw_backend_report_format.clone()),
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
                bijux_dna_stages_fastq::observer::parse_merge_pairs_report(&raw).ok()
            });
            Some(serde_json::json!({
                "schema_version": "bijux.fastq.merge_pairs.extra_artifacts.v2",
                "stage": stage_id,
                "paired_mode": governed.as_ref().map(|report| report.paired_mode.clone()),
                "merge_engine": governed.as_ref().map(|report| report.merge_engine.clone()),
                "merge_overlap": governed.as_ref().and_then(|report| report.merge_overlap),
                "min_length": governed.as_ref().and_then(|report| report.min_len),
                "unmerged_read_policy": governed.as_ref().map(|report| report.unmerged_read_policy.clone()),
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
                bijux_dna_stages_fastq::observer::parse_extract_umis_report(&raw).ok()
            });
            Some(serde_json::json!({
                "schema_version": "bijux.fastq.extract_umis.extra_artifacts.v2",
                "stage": stage_id,
                "tool": governed.as_ref().map(|report| report.tool_id.clone()),
                "paired_mode": governed.as_ref().map(|report| report.paired_mode),
                "threads": governed.as_ref().map(|report| report.threads),
                "umi_pattern": governed.as_ref().map(|report| report.umi_pattern.clone()),
                "reads_with_umi": governed.as_ref().map(|report| report.reads_with_umi),
                "raw_backend_report": governed.as_ref().and_then(|report| report.raw_backend_report.clone()),
                "raw_backend_report_format": governed.as_ref().and_then(|report| report.raw_backend_report_format.clone()),
                "report_json": report_path,
            }))
        }
        "fastq.screen_taxonomy" => {
            let report_path = discover_screen_taxonomy_report_path(stage_root, &execution.outputs)
                .unwrap_or_else(|| stage_root.join("classification_report.json"));
            let governed = std::fs::read_to_string(&report_path).ok().and_then(|raw| {
                bijux_dna_stages_fastq::observer::parse_screen_taxonomy_report(&raw).ok()
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
                bijux_dna_stages_fastq::observer::parse_report_qc_report(&raw).ok()
            });
            Some(serde_json::json!({
                "schema_version": "bijux.fastq.report_qc.extra_artifacts.v2",
                "stage": stage_id,
                "aggregation_engine": governed.as_ref().map(|report| report.aggregation_engine.clone()),
                "aggregation_scope": governed.as_ref().map(|report| report.aggregation_scope.clone()),
                "paired_mode": governed.as_ref().map(|report| report.paired_mode.clone()),
                "governed_qc_input_count": governed.as_ref().map(|report| report.governed_qc_input_count),
                "governed_qc_contributor_stage_ids": governed.as_ref().map(|report| report.governed_qc_contributor_stage_ids.clone()),
                "governed_qc_contributor_tool_ids": governed.as_ref().map(|report| report.governed_qc_contributor_tool_ids.clone()),
                "governed_qc_lineage_hash": governed.as_ref().and_then(|report| report.governed_qc_lineage_hash.clone()),
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
                bijux_dna_stages_fastq::observer::parse_remove_duplicates_report(&raw).ok()
            });
            Some(serde_json::json!({
                "schema_version": "bijux.fastq.remove_duplicates.extra_artifacts.v2",
                "stage": stage_id,
                "paired_mode": governed.as_ref().map(|report| report.paired_mode.clone()),
                "dedup_mode": governed.as_ref().map(|report| report.dedup_mode.clone()),
                "keep_order": governed.as_ref().map(|report| report.keep_order),
                "pair_count_match": governed.as_ref().and_then(|report| report.pair_count_match),
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
                bijux_dna_stages_fastq::observer::parse_deplete_reference_contaminants_report(&raw)
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
                "index_artifact": governed.as_ref().map(|report| report.index_artifact.clone()),
                "reference_index_backend": governed.as_ref().map(|report| report.reference_index_backend.clone()),
                "reference_build_id": governed.as_ref().and_then(|report| report.reference_build_id.clone()),
                "reference_digest": governed.as_ref().and_then(|report| report.reference_digest.clone()),
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
                bijux_dna_stages_fastq::observer::parse_deplete_rrna_report(&raw).ok()
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
                "screening_engine": governed.as_ref().map(|report| report.screening_engine.clone()),
                "report_format": governed.as_ref().map(|report| report.report_format.clone()),
                "min_identity": governed.as_ref().and_then(|report| report.min_identity),
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
                bijux_dna_stages_fastq::observer::parse_deplete_host_report(&raw).ok()
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
                bijux_dna_stages_fastq::observer::parse_normalize_primers_report(&raw).ok()
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
                bijux_dna_stages_fastq::observer::parse_terminal_damage_report(&raw).ok()
            });
            Some(serde_json::json!({
                "schema_version": "bijux.fastq.trim_terminal_damage.v2",
                "stage": stage_id,
                "damage_mode": governed.as_ref().map(|report| report.damage_mode),
                "execution_policy": governed.as_ref().map(|report| report.execution_policy),
                "requested_trim_5p_bases": governed.as_ref().and_then(|report| report.requested_trim_5p_bases),
                "requested_trim_3p_bases": governed.as_ref().and_then(|report| report.requested_trim_3p_bases),
                "udg_classification": governed.as_ref().map(|report| report.udg_classification.clone()),
                "raw_backend_report_format": governed.as_ref().and_then(|report| report.raw_backend_report_format.clone()),
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
                bijux_dna_stages_fastq::observer::parse_remove_chimeras_report(&raw).ok()
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
        "fastq.cluster_otus" => Some(serde_json::json!({
            "schema_version": "bijux.fastq.cluster_otus.v1",
            "stage": stage_id,
            "applicability": "edna_pollen_only",
        })),
        "fastq.infer_asvs" => {
            let report_path = stage_root.join("infer_asvs_report.json");
            let governed = std::fs::read_to_string(&report_path).ok().and_then(|raw| {
                bijux_dna_stages_fastq::observer::parse_infer_asvs_report(&raw).ok()
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
                bijux_dna_stages_fastq::observer::parse_normalize_abundance_report(&raw).ok()
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

fn write_stage_standardized_metrics(
    stage_root: &std::path::Path,
    stage_id: &str,
    out_dir: &std::path::Path,
    execution: &StageResultV1,
) -> Result<()> {
    let metrics = match stage_id {
        "fastq.index_reference" => parse_index_reference_metrics(out_dir),
        "fastq.validate_reads" => parse_validate_reads_metrics(out_dir, execution),
        "fastq.detect_adapters" => parse_detect_adapters_metrics(out_dir),
        "fastq.profile_read_lengths" => parse_profile_read_lengths_metrics(out_dir),
        "fastq.profile_overrepresented_sequences" => parse_profile_overrepresented_metrics(out_dir),
        "fastq.trim_polyg_tails" => parse_trim_polyg_metrics(out_dir),
        "fastq.screen_taxonomy" => parse_screen_taxonomy_metrics(out_dir),
        "fastq.filter_low_complexity" => parse_filter_low_complexity_metrics(out_dir),
        "fastq.trim_reads" => parse_trim_reads_metrics(out_dir),
        "fastq.filter_reads" => parse_filter_reads_metrics(out_dir),
        "fastq.correct_errors" => serde_json::json!({
            "schema_version": "bijux.fastq_stage_metrics.v1",
            "stage": stage_id,
            "fields": ["reads_in", "reads_out", "bases_corrected", "substitutions_corrected"],
            "report_json": out_dir.join("correct_report.json"),
        }),
        "fastq.merge_pairs" => parse_merge_pairs_metrics(out_dir),
        "fastq.remove_duplicates" => parse_remove_duplicates_metrics(out_dir),
        "fastq.extract_umis" => parse_extract_umis_metrics(out_dir),
        "fastq.deplete_host" => parse_deplete_host_metrics(out_dir),
        "fastq.deplete_reference_contaminants" => {
            parse_deplete_reference_contaminants_metrics(out_dir)
        }
        "fastq.deplete_rrna" => parse_deplete_rrna_metrics(out_dir),
        "fastq.profile_reads" => parse_profile_reads_metrics(out_dir),
        "fastq.report_qc" => parse_report_qc_metrics(out_dir),
        "fastq.normalize_primers" => parse_normalize_primers_metrics(out_dir),
        "fastq.normalize_abundance" => parse_normalize_abundance_metrics(out_dir),
        "fastq.trim_terminal_damage" => parse_trim_terminal_damage_metrics(out_dir),
        "fastq.remove_chimeras" => parse_remove_chimeras_metrics(out_dir),
        "fastq.infer_asvs" => parse_infer_asvs_metrics(out_dir),
        "fastq.cluster_otus" => serde_json::json!({
            "schema_version": "bijux.fastq_stage_metrics.v1",
            "stage": stage_id,
            "fields": ["otu_count", "cluster_radius", "sample_count"],
        }),
        _ => return Ok(()),
    };
    bijux_dna_infra::atomic_write_json(
        &stage_root.join("stage.metrics.standardized.json"),
        &metrics,
    )
    .context("write standardized stage metrics")
}

fn discover_screen_taxonomy_report_path(
    stage_root: &std::path::Path,
    outputs: &[std::path::PathBuf],
) -> Option<std::path::PathBuf> {
    outputs
        .iter()
        .find(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(".classifications.json"))
        })
        .cloned()
        .or_else(|| {
            [
                "kraken2.classifications.json",
                "krakenuniq.classifications.json",
                "centrifuge.classifications.json",
                "kaiju.classifications.json",
                "classification_report.json",
            ]
            .into_iter()
            .map(|name| stage_root.join(name))
            .find(|path| path.exists())
        })
}
