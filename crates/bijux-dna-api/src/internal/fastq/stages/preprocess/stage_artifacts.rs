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
        "fastq.trim_polyg_tails" => Some(serde_json::json!({
            "schema_version": "bijux.fastq.polyg.v1",
            "stage": stage_id,
            "before_after_distribution": {
                "before_polyg_reads": parse_first_u64_after_key(&execution.stdout, "polyG before"),
                "after_polyg_reads": parse_first_u64_after_key(&execution.stdout, "polyG after"),
            }
        })),
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
        "fastq.normalize_primers" => Some(serde_json::json!({
            "schema_version": "bijux.fastq.normalize_primers.v1",
            "stage": stage_id,
            "orientation_policy": "enforced_by_tool_backend",
            "mismatch_policy": "configured_in_stage_params",
        })),
        "fastq.trim_terminal_damage" => Some(serde_json::json!({
            "schema_version": "bijux.fastq.trim_terminal_damage.v1",
            "stage": stage_id,
            "policy": "mask_or_trim_terminal_bases",
            "udg_classification_source": "configured_or_inferred",
        })),
        "fastq.remove_chimeras" => Some(serde_json::json!({
            "schema_version": "bijux.fastq.remove_chimeras.v1",
            "stage": stage_id,
            "chimera_removed": parse_first_u64_after_key(&execution.stderr, "chimera"),
        })),
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
        "fastq.normalize_abundance" => Some(serde_json::json!({
            "schema_version": "bijux.fastq.normalize_abundance.v1",
            "stage": stage_id,
            "normalized_table_emitted": true,
        })),
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
        "fastq.validate_reads" => parse_validate_reads_metrics(execution),
        "fastq.detect_adapters" => serde_json::json!({
            "schema_version": "bijux.fastq_stage_metrics.v1",
            "stage": stage_id,
            "adapter_inference": parse_detect_adapters_metrics(out_dir).get("adapter_inference").cloned().unwrap_or_else(|| serde_json::json!({})),
        }),
        "fastq.profile_read_lengths" => serde_json::json!({
            "schema_version": "bijux.fastq_stage_metrics.v1",
            "stage": stage_id,
            "fields": ["sample_id", "read_length", "count"],
            "tsv_path": out_dir.join("length_distribution.tsv"),
            "json_path": out_dir.join("length_distribution.json"),
        }),
        "fastq.profile_overrepresented_sequences" => serde_json::json!({
            "schema_version": "bijux.fastq_stage_metrics.v1",
            "stage": stage_id,
            "fields": ["sequence", "count", "fraction", "flag"],
            "tsv_path": out_dir.join("overrepresented_sequences.tsv"),
            "json_path": out_dir.join("overrepresented_sequences.json"),
        }),
        "fastq.trim_polyg_tails" => serde_json::json!({
            "schema_version": "bijux.fastq_stage_metrics.v1",
            "stage": stage_id,
            "applicability": {
                "requires_illumina_like_cycle_artifacts": true,
            },
            "report_json": out_dir.join("polyg_tailing_report.json"),
        }),
        "fastq.filter_low_complexity" => serde_json::json!({
            "schema_version": "bijux.fastq_stage_metrics.v1",
            "stage": stage_id,
            "filter_counts": {
                "filtered_reads": parse_low_complexity_filtered_count(&execution.stdout, &execution.stderr),
            },
            "report_json": out_dir.join("low_complexity_report.json"),
        }),
        "fastq.trim_reads" => serde_json::json!({
            "schema_version": "bijux.fastq_stage_metrics.v1",
            "stage": stage_id,
            "fields": ["reads_in", "reads_out", "bases_in", "bases_out"],
            "report_json": out_dir.join("trim_report.json"),
        }),
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
        "fastq.remove_duplicates" => serde_json::json!({
            "schema_version": "bijux.fastq_stage_metrics.v1",
            "stage": stage_id,
            "fields": ["reads_in", "reads_out", "duplicate_reads"],
            "report_json": out_dir.join("deduplicate_report.json"),
        }),
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
        "fastq.screen_taxonomy" => serde_json::json!({
            "schema_version": "bijux.fastq_stage_metrics.v1",
            "stage": stage_id,
            "fields": ["classified_reads", "unclassified_reads", "top_taxa"],
            "report_tsv": out_dir.join("screen_report.tsv"),
            "report_json": out_dir.join("classification.report.json"),
        }),
        "fastq.report_qc" => serde_json::json!({
            "schema_version": "bijux.fastq_stage_metrics.v1",
            "stage": stage_id,
            "fields": ["qc_modules", "warnings", "failures"],
            "report_html": out_dir.join("multiqc").join("multiqc_report.html"),
            "report_data_dir": out_dir.join("multiqc").join("multiqc_data"),
        }),
        "fastq.normalize_primers" => serde_json::json!({
            "schema_version": "bijux.fastq_stage_metrics.v1",
            "stage": stage_id,
            "fields": ["reads_in", "reads_out", "primer_trimmed_reads"],
        }),
        "fastq.trim_terminal_damage" => serde_json::json!({
            "schema_version": "bijux.fastq_stage_metrics.v1",
            "stage": stage_id,
            "fields": [
                "udg_classification",
                "terminal_base_composition_pre",
                "terminal_base_composition_post",
                "ct_ga_asymmetry_pre",
                "ct_ga_asymmetry_post"
            ],
        }),
        "fastq.remove_chimeras" => serde_json::json!({
            "schema_version": "bijux.fastq_stage_metrics.v1",
            "stage": stage_id,
            "fields": ["reads_in", "reads_out", "chimeras_removed"],
        }),
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
        "fastq.normalize_abundance" => serde_json::json!({
            "schema_version": "bijux.fastq_stage_metrics.v1",
            "stage": stage_id,
            "fields": ["table_rows", "sample_count", "normalization_method"],
        }),
        _ => return Ok(()),
    };
    bijux_dna_infra::atomic_write_json(&stage_root.join("stage.metrics.standardized.json"), &metrics)
        .context("write standardized stage metrics")
}
