fn emit_fastq_stage_extra_artifacts(
    stage_root: &std::path::Path,
    stage_id: &str,
    execution: &StageResultV1,
) -> Result<()> {
    let payload = match stage_id {
        "fastq.filter_reads" => Some(serde_json::json!({
            "schema_version": "bijux.fastq.filter_reads_reasons.v1",
            "stage": stage_id,
            "reasons": {
                "low_quality": parse_first_u64_after_key(&execution.stderr, "low quality"),
                "too_short": parse_first_u64_after_key(&execution.stderr, "too short"),
                "too_many_n": parse_first_u64_after_key(&execution.stderr, "N"),
            }
        })),
        "fastq.filter_low_complexity" => Some(serde_json::json!({
            "schema_version": "bijux.fastq.filter_low_complexity.v1",
            "stage": stage_id,
            "removed_reads": parse_low_complexity_filtered_count(&execution.stdout, &execution.stderr),
        })),
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
                    path.file_name().and_then(|name| name.to_str())
                        == Some("report_qc_report.json")
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
        "fastq.deplete_reference_contaminants" => Some(serde_json::json!({
            "schema_version": "bijux.fastq.deplete_reference_contaminants.v1",
            "stage": stage_id,
            "bank_usage": "assets/reference contaminant bank required",
        })),
        "fastq.deplete_rrna" => Some(serde_json::json!({
            "schema_version": "bijux.fastq.deplete_rrna.v1",
            "stage": stage_id,
            "db_governance": "explicit local sortmerna db required",
        })),
        "fastq.deplete_host" => Some(serde_json::json!({
            "schema_version": "bijux.fastq.deplete_host.v1",
            "stage": stage_id,
            "reference_resolution": "explicit host reference required via planned command inputs",
        })),
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
        "fastq.infer_asvs" => Some(serde_json::json!({
            "schema_version": "bijux.fastq.infer_asvs.v1",
            "stage": stage_id,
            "runtime_contract": "R_runtime_required",
            "applicability": "edna_pollen_only",
        })),
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
        "fastq.validate_reads" => parse_validate_reads_metrics(out_dir, execution),
        "fastq.detect_adapters" => serde_json::json!({
            "schema_version": "bijux.fastq_stage_metrics.v1",
            "stage": stage_id,
            "report_only": true,
            "adapter_inference": parse_detect_adapters_metrics(out_dir).get("adapter_inference").cloned().unwrap_or_else(|| serde_json::json!({})),
        }),
        "fastq.profile_read_lengths" => parse_profile_read_lengths_metrics(out_dir),
        "fastq.profile_overrepresented_sequences" => parse_profile_overrepresented_metrics(out_dir),
        "fastq.trim_polyg_tails" => parse_trim_polyg_metrics(out_dir),
        "fastq.screen_taxonomy" => parse_screen_taxonomy_metrics(out_dir),
        "fastq.filter_low_complexity" => serde_json::json!({
            "schema_version": "bijux.fastq_stage_metrics.v1",
            "stage": stage_id,
            "filter_counts": {
                "filtered_reads": parse_low_complexity_filtered_count(&execution.stdout, &execution.stderr),
            },
            "report_json": out_dir.join("low_complexity_report.json"),
        }),
        "fastq.trim_reads" => parse_trim_reads_metrics(out_dir),
        "fastq.filter_reads" => serde_json::json!({
            "schema_version": "bijux.fastq_stage_metrics.v1",
            "stage": stage_id,
            "fields": ["reads_in", "reads_out", "filtered_low_quality", "filtered_too_short", "filtered_n_content"],
            "report_json": out_dir.join("filter_report.json"),
        }),
        "fastq.correct_errors" => serde_json::json!({
            "schema_version": "bijux.fastq_stage_metrics.v1",
            "stage": stage_id,
            "fields": ["reads_in", "reads_out", "bases_corrected", "substitutions_corrected"],
            "report_json": out_dir.join("correct_report.json"),
        }),
        "fastq.merge_pairs" => serde_json::json!({
            "schema_version": "bijux.fastq_stage_metrics.v1",
            "stage": stage_id,
            "fields": ["pairs_in", "pairs_merged", "pairs_unmerged"],
            "report_json": out_dir.join("merge_report.json"),
        }),
        "fastq.remove_duplicates" => parse_remove_duplicates_metrics(out_dir),
        "fastq.extract_umis" => serde_json::json!({
            "schema_version": "bijux.fastq_stage_metrics.v1",
            "stage": stage_id,
            "fields": ["reads_in", "reads_out", "umi_groups", "umi_collisions"],
            "report_json": out_dir.join("umi_report.json"),
        }),
        "fastq.deplete_host" => serde_json::json!({
            "schema_version": "bijux.fastq_stage_metrics.v1",
            "stage": stage_id,
            "fields": ["reads_in", "reads_unmapped_out", "host_mapped_reads"],
            "report_json": out_dir.join("host_depletion_report.json"),
        }),
        "fastq.deplete_reference_contaminants" => serde_json::json!({
            "schema_version": "bijux.fastq_stage_metrics.v1",
            "stage": stage_id,
            "fields": ["reads_in", "reads_out", "contaminant_mapped_reads"],
            "report_json": out_dir.join("contaminant_screen_report.json"),
        }),
        "fastq.deplete_rrna" => serde_json::json!({
            "schema_version": "bijux.fastq_stage_metrics.v1",
            "stage": stage_id,
            "fields": ["reads_in", "rrna_hits", "rrna_fraction"],
            "report_tsv": out_dir.join("rrna_report.tsv"),
            "report_json": out_dir.join("rrna_report.json"),
        }),
        "fastq.profile_reads" => parse_profile_reads_metrics(out_dir),
        "fastq.report_qc" => parse_report_qc_metrics(out_dir),
        "fastq.normalize_primers" => parse_normalize_primers_metrics(out_dir),
        "fastq.normalize_abundance" => parse_normalize_abundance_metrics(out_dir),
        "fastq.trim_terminal_damage" => parse_trim_terminal_damage_metrics(out_dir),
        "fastq.remove_chimeras" => parse_remove_chimeras_metrics(out_dir),
        "fastq.infer_asvs" => serde_json::json!({
            "schema_version": "bijux.fastq_stage_metrics.v1",
            "stage": stage_id,
            "fields": ["asv_count", "nonchimera_reads", "sample_count"],
        }),
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
