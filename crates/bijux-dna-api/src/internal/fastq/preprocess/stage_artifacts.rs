fn emit_fastq_stage_extra_artifacts(
    stage_root: &std::path::Path,
    stage_id: &str,
    execution: &StageResultV1,
) -> Result<()> {
    let payload = match stage_id {
        "fastq.filter" => Some(serde_json::json!({
            "schema_version": "bijux.fastq.filter_reasons.v1",
            "stage": stage_id,
            "reasons": {
                "low_quality": parse_first_u64_after_key(&execution.stderr, "low quality"),
                "too_short": parse_first_u64_after_key(&execution.stderr, "too short"),
                "too_many_n": parse_first_u64_after_key(&execution.stderr, "N"),
            }
        })),
        "fastq.low_complexity" => Some(serde_json::json!({
            "schema_version": "bijux.fastq.low_complexity.v1",
            "stage": stage_id,
            "removed_reads": parse_low_complexity_filtered_count(&execution.stdout, &execution.stderr),
        })),
        "fastq.polyg_tailing" => Some(serde_json::json!({
            "schema_version": "bijux.fastq.polyg.v1",
            "stage": stage_id,
            "before_after_distribution": {
                "before_polyg_reads": parse_first_u64_after_key(&execution.stdout, "polyG before"),
                "after_polyg_reads": parse_first_u64_after_key(&execution.stdout, "polyG after"),
            }
        })),
        "fastq.contaminant_screen" => Some(serde_json::json!({
            "schema_version": "bijux.fastq.contaminant_screen.v1",
            "stage": stage_id,
            "bank_usage": "assets/reference contaminant bank required",
        })),
        "fastq.rrna" => Some(serde_json::json!({
            "schema_version": "bijux.fastq.rrna.v1",
            "stage": stage_id,
            "db_governance": "explicit local sortmerna db required",
        })),
        "fastq.host_depletion" => Some(serde_json::json!({
            "schema_version": "bijux.fastq.host_depletion.v1",
            "stage": stage_id,
            "reference_resolution": "explicit host reference required via planned command inputs",
        })),
        "fastq.primer_normalization" => Some(serde_json::json!({
            "schema_version": "bijux.fastq.primer_normalization.v1",
            "stage": stage_id,
            "orientation_policy": "enforced_by_tool_backend",
            "mismatch_policy": "configured_in_stage_params",
        })),
        "fastq.damage_aware_pretrim" => Some(serde_json::json!({
            "schema_version": "bijux.fastq.damage_aware_pretrim.v1",
            "stage": stage_id,
            "policy": "mask_or_trim_terminal_bases",
            "udg_classification_source": "configured_or_inferred",
        })),
        "fastq.chimera_detection" => Some(serde_json::json!({
            "schema_version": "bijux.fastq.chimera_detection.v1",
            "stage": stage_id,
            "chimera_removed": parse_first_u64_after_key(&execution.stderr, "chimera"),
        })),
        "fastq.otu_clustering" => Some(serde_json::json!({
            "schema_version": "bijux.fastq.otu_clustering.v1",
            "stage": stage_id,
            "applicability": "edna_pollen_only",
        })),
        "fastq.asv_inference" => Some(serde_json::json!({
            "schema_version": "bijux.fastq.asv_inference.v1",
            "stage": stage_id,
            "runtime_contract": "R_runtime_required",
            "applicability": "edna_pollen_only",
        })),
        "fastq.abundance_normalization" => Some(serde_json::json!({
            "schema_version": "bijux.fastq.abundance_normalization.v1",
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
        "fastq.validate_pre" => parse_validate_pre_metrics(execution),
        "fastq.detect_adapters" => serde_json::json!({
            "schema_version": "bijux.fastq_stage_metrics.v1",
            "stage": stage_id,
            "adapter_inference": parse_detect_adapters_metrics(out_dir).get("adapter_inference").cloned().unwrap_or_else(|| serde_json::json!({})),
        }),
        "fastq.length_distribution_pre" => serde_json::json!({
            "schema_version": "bijux.fastq_stage_metrics.v1",
            "stage": stage_id,
            "fields": ["sample_id", "read_length", "count"],
            "tsv_path": out_dir.join("length_distribution.tsv"),
            "json_path": out_dir.join("length_distribution.json"),
        }),
        "fastq.overrepresented_sequences" => serde_json::json!({
            "schema_version": "bijux.fastq_stage_metrics.v1",
            "stage": stage_id,
            "fields": ["sequence", "count", "fraction", "flag"],
            "tsv_path": out_dir.join("overrepresented_sequences.tsv"),
            "json_path": out_dir.join("overrepresented_sequences.json"),
        }),
        "fastq.polyg_tailing" => serde_json::json!({
            "schema_version": "bijux.fastq_stage_metrics.v1",
            "stage": stage_id,
            "applicability": {
                "requires_illumina_like_cycle_artifacts": true,
            },
            "report_json": out_dir.join("polyg_tailing_report.json"),
        }),
        "fastq.low_complexity" => serde_json::json!({
            "schema_version": "bijux.fastq_stage_metrics.v1",
            "stage": stage_id,
            "filter_counts": {
                "filtered_reads": parse_low_complexity_filtered_count(&execution.stdout, &execution.stderr),
            },
            "report_json": out_dir.join("low_complexity_report.json"),
        }),
        "fastq.trim" => serde_json::json!({
            "schema_version": "bijux.fastq_stage_metrics.v1",
            "stage": stage_id,
            "fields": ["reads_in", "reads_out", "bases_in", "bases_out"],
            "report_json": out_dir.join("trim_report.json"),
        }),
        "fastq.filter" => serde_json::json!({
            "schema_version": "bijux.fastq_stage_metrics.v1",
            "stage": stage_id,
            "fields": ["reads_in", "reads_out", "filtered_low_quality", "filtered_too_short", "filtered_n_content"],
            "report_json": out_dir.join("filter_report.json"),
        }),
        "fastq.correct" => serde_json::json!({
            "schema_version": "bijux.fastq_stage_metrics.v1",
            "stage": stage_id,
            "fields": ["reads_in", "reads_out", "bases_corrected", "substitutions_corrected"],
            "report_json": out_dir.join("correct_report.json"),
        }),
        "fastq.merge" => serde_json::json!({
            "schema_version": "bijux.fastq_stage_metrics.v1",
            "stage": stage_id,
            "fields": ["pairs_in", "pairs_merged", "pairs_unmerged"],
            "report_json": out_dir.join("merge_report.json"),
        }),
        "fastq.deduplicate" => serde_json::json!({
            "schema_version": "bijux.fastq_stage_metrics.v1",
            "stage": stage_id,
            "fields": ["reads_in", "reads_out", "duplicate_reads"],
            "report_json": out_dir.join("deduplicate_report.json"),
        }),
        "fastq.umi" => serde_json::json!({
            "schema_version": "bijux.fastq_stage_metrics.v1",
            "stage": stage_id,
            "fields": ["reads_in", "reads_out", "umi_groups", "umi_collisions"],
            "report_json": out_dir.join("umi_report.json"),
        }),
        "fastq.host_depletion" => serde_json::json!({
            "schema_version": "bijux.fastq_stage_metrics.v1",
            "stage": stage_id,
            "fields": ["reads_in", "reads_unmapped_out", "host_mapped_reads"],
            "report_json": out_dir.join("host_depletion_report.json"),
        }),
        "fastq.contaminant_screen" => serde_json::json!({
            "schema_version": "bijux.fastq_stage_metrics.v1",
            "stage": stage_id,
            "fields": ["reads_in", "reads_out", "contaminant_mapped_reads"],
            "report_json": out_dir.join("contaminant_screen_report.json"),
        }),
        "fastq.rrna" => serde_json::json!({
            "schema_version": "bijux.fastq_stage_metrics.v1",
            "stage": stage_id,
            "fields": ["reads_in", "rrna_hits", "rrna_fraction"],
            "report_tsv": out_dir.join("rrna_report.tsv"),
            "report_json": out_dir.join("rrna_report.json"),
        }),
        "fastq.screen" => serde_json::json!({
            "schema_version": "bijux.fastq_stage_metrics.v1",
            "stage": stage_id,
            "fields": ["classified_reads", "unclassified_reads", "top_taxa"],
            "report_tsv": out_dir.join("screen_report.tsv"),
            "report_json": out_dir.join("classification.report.json"),
        }),
        "fastq.qc_post" => serde_json::json!({
            "schema_version": "bijux.fastq_stage_metrics.v1",
            "stage": stage_id,
            "fields": ["qc_modules", "warnings", "failures"],
            "report_html": out_dir.join("multiqc").join("multiqc_report.html"),
            "report_data_dir": out_dir.join("multiqc").join("multiqc_data"),
        }),
        "fastq.primer_normalization" => serde_json::json!({
            "schema_version": "bijux.fastq_stage_metrics.v1",
            "stage": stage_id,
            "fields": ["reads_in", "reads_out", "primer_trimmed_reads"],
        }),
        "fastq.damage_aware_pretrim" => serde_json::json!({
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
        "fastq.chimera_detection" => serde_json::json!({
            "schema_version": "bijux.fastq_stage_metrics.v1",
            "stage": stage_id,
            "fields": ["reads_in", "reads_out", "chimeras_removed"],
        }),
        "fastq.asv_inference" => serde_json::json!({
            "schema_version": "bijux.fastq_stage_metrics.v1",
            "stage": stage_id,
            "fields": ["asv_count", "nonchimera_reads", "sample_count"],
        }),
        "fastq.otu_clustering" => serde_json::json!({
            "schema_version": "bijux.fastq_stage_metrics.v1",
            "stage": stage_id,
            "fields": ["otu_count", "cluster_radius", "sample_count"],
        }),
        "fastq.abundance_normalization" => serde_json::json!({
            "schema_version": "bijux.fastq_stage_metrics.v1",
            "stage": stage_id,
            "fields": ["table_rows", "sample_count", "normalization_method"],
        }),
        _ => return Ok(()),
    };
    bijux_dna_infra::atomic_write_json(&stage_root.join("stage.metrics.standardized.json"), &metrics)
        .context("write standardized stage metrics")
}
