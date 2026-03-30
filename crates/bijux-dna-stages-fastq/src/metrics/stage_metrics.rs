use super::envelope_support::{
    pair_counts_from_paths, path_from_params, stats_for_paths, zero_seqkit_metrics,
};
use super::stage_metrics_reporting::stage_metrics_for_stage as reporting_stage_metrics_for_stage;
use super::stage_metrics_transform::stage_metrics_for_stage as transform_stage_metrics_for_stage;
use super::*;

pub(super) fn stage_metrics_for_plan(
    plan: &StagePlanV1,
    inputs: &[PathBuf],
    outputs: &[PathBuf],
) -> Result<serde_json::Value> {
    let mut metrics = match plan.stage_id.as_str() {
        id_catalog::FASTQ_TRIM
        | id_catalog::FASTQ_FILTER
        | id_catalog::FASTQ_DEDUPLICATE
        | id_catalog::FASTQ_LOW_COMPLEXITY
        | id_catalog::FASTQ_MERGE
        | id_catalog::FASTQ_VALIDATE_PRE => {
            match transform_stage_metrics_for_stage(plan, inputs, outputs) {
                Some(metrics) => metrics,
                None => unreachable!("transform stage ids must be handled"),
            }
        }
        "fastq.normalize_primers"
        | "fastq.profile_overrepresented_sequences"
        | id_catalog::FASTQ_DETECT_ADAPTERS
        | id_catalog::FASTQ_CORRECT
        | id_catalog::FASTQ_UMI
        | id_catalog::FASTQ_PREPROCESS
        | id_catalog::FASTQ_QC_POST
        | id_catalog::FASTQ_STATS_NEUTRAL
        | "fastq.profile_read_lengths" => {
            match reporting_stage_metrics_for_stage(plan, inputs, outputs) {
                Some(metrics) => metrics,
                None => unreachable!("reporting stage ids must be handled"),
            }
        }
        "fastq.normalize_abundance" => {
            let report_path = path_from_params(&plan.params, "report_json")
                .or_else(|| {
                    plan.io
                        .outputs
                        .iter()
                        .find(|artifact| artifact.name.as_str() == "report_json")
                        .map(|artifact| artifact.path.clone())
                })
                .or_else(|| {
                    let fallback = plan.out_dir.join("normalize_abundance_report.json");
                    fallback.exists().then_some(fallback)
                });
            let governed_report = report_path
                .and_then(|path| std::fs::read_to_string(&path).ok())
                .and_then(|raw| crate::observer::parse_normalize_abundance_report(&raw).ok());
            if let Some(report) = governed_report {
                serde_json::json!({
                    "table_rows": report.table_rows,
                    "sample_count": report.sample_count,
                    "feature_count": report.feature_count,
                    "zero_fraction": report.zero_fraction,
                    "method": report.method,
                    "input_value_column": report.input_value_column,
                    "normalized_value_column": report.normalized_value_column,
                    "compositional_rule": report.compositional_rule,
                    "scale_factor": report.scale_factor,
                    "per_sample_sums": report.per_sample_sums,
                })
            } else {
                serde_json::json!({})
            }
        }
        "fastq.infer_asvs" => {
            let report_path = path_from_params(&plan.params, "report_json")
                .or_else(|| {
                    plan.io
                        .outputs
                        .iter()
                        .find(|artifact| artifact.name.as_str() == "report_json")
                        .map(|artifact| artifact.path.clone())
                })
                .or_else(|| {
                    let fallback = plan.out_dir.join("infer_asvs_report.json");
                    fallback.exists().then_some(fallback)
                });
            let governed_report = report_path
                .and_then(|path| std::fs::read_to_string(&path).ok())
                .and_then(|raw| crate::observer::parse_infer_asvs_report(&raw).ok());
            if let Some(report) = governed_report {
                serde_json::json!({
                    "asv_count": report.asv_count,
                    "sample_count": report.sample_count,
                    "representative_sequence_count": report.representative_sequence_count,
                    "paired_mode": report.paired_mode,
                    "denoising_method": report.denoising_method,
                    "pooling_mode": report.pooling_mode,
                    "chimera_policy": report.chimera_policy,
                    "output_table_kind": report.output_table_kind,
                    "used_fallback": report.used_fallback,
                    "raw_backend_report_format": report.raw_backend_report_format,
                })
            } else {
                serde_json::json!({})
            }
        }
        "fastq.cluster_otus" => {
            let report_path = path_from_params(&plan.params, "report_json")
                .or_else(|| {
                    plan.io
                        .outputs
                        .iter()
                        .find(|artifact| artifact.name.as_str() == "report_json")
                        .map(|artifact| artifact.path.clone())
                })
                .or_else(|| {
                    let fallback = plan.out_dir.join("cluster_otus_report.json");
                    fallback.exists().then_some(fallback)
                });
            let governed_report = report_path
                .and_then(|path| std::fs::read_to_string(&path).ok())
                .and_then(|raw| crate::observer::parse_cluster_otus_report(&raw).ok());
            if let Some(report) = governed_report {
                serde_json::json!({
                    "otu_identity": report.otu_identity,
                    "threads": report.threads,
                    "otu_count": report.otu_count,
                    "sample_count": report.sample_count,
                    "representative_sequence_count": report.representative_sequence_count,
                    "output_table_kind": report.output_table_kind,
                    "used_fallback": report.used_fallback,
                    "raw_backend_report_format": report.raw_backend_report_format,
                })
            } else {
                serde_json::json!({})
            }
        }
        "fastq.index_reference" => {
            let report_path = path_from_params(&plan.params, "report_json")
                .or_else(|| {
                    plan.io
                        .outputs
                        .iter()
                        .find(|artifact| artifact.name.as_str() == "report_json")
                        .map(|artifact| artifact.path.clone())
                })
                .or_else(|| {
                    let fallback = plan.out_dir.join("index_reference_report.json");
                    fallback.exists().then_some(fallback)
                });
            let governed_report = report_path
                .and_then(|path| std::fs::read_to_string(&path).ok())
                .and_then(|raw| crate::observer::parse_index_reference_report(&raw).ok());
            if let Some(report) = governed_report {
                serde_json::json!({
                    "threads": report.threads,
                    "index_format": report.index_format,
                    "reference_bytes": report.reference_bytes,
                    "index_bytes": report.index_bytes,
                    "index_file_count": report.index_file_count,
                    "index_prefix": report.index_prefix,
                    "emitted_file_count": report.emitted_files.len(),
                })
            } else {
                serde_json::json!({})
            }
        }
        id_catalog::FASTQ_SCREEN => {
            let governed_report_path = path_from_params(&plan.params, "assignments")
                .or_else(|| {
                    plan.io
                        .outputs
                        .iter()
                        .find(|artifact| artifact.name.as_str() == "classification_report_json")
                        .map(|artifact| artifact.path.clone())
                })
                .or_else(|| {
                    let fallback = plan.out_dir.join("kraken2.classifications.json");
                    fallback.exists().then_some(fallback)
                });
            let governed_report = governed_report_path
                .and_then(|path| std::fs::read_to_string(&path).ok())
                .and_then(|raw| crate::observer::parse_screen_taxonomy_report(&raw).ok());
            if let Some(report) = governed_report {
                let contamination_summary = report
                    .summary_entries
                    .iter()
                    .map(|entry| {
                        serde_json::json!({
                            "label": entry.label,
                            "percent": entry.percent,
                        })
                    })
                    .collect::<Vec<_>>();
                serde_json::json!({
                    "reads_in": report.reads_in,
                    "reads_out": report.reads_out,
                    "bases_in": report.bases_in,
                    "bases_out": report.bases_out,
                    "pairs_in": report.pairs_in,
                    "pairs_out": report.pairs_out,
                    "contamination_rate": report.contamination_rate,
                    "classified_fraction": report.classified_fraction,
                    "unclassified_fraction": report.unclassified_fraction,
                    "contamination_summary": contamination_summary,
                    "top_taxa": report.top_taxa,
                    "database_artifact_id": report.database_artifact_id,
                    "classifier": report.classifier,
                })
            } else {
                let stats = stats_for_paths(&[inputs.first().map(PathBuf::as_path)])?;
                let input = stats.first().copied().unwrap_or_else(zero_seqkit_metrics);
                let output = if outputs.is_empty() {
                    input
                } else {
                    let stats = stats_for_paths(&[outputs.first().map(PathBuf::as_path)])?;
                    stats.first().copied().unwrap_or_else(zero_seqkit_metrics)
                };
                let (pairs_in, pairs_out) = pair_counts_from_paths(inputs, outputs)?;
                let report_path = path_from_params(&plan.params, "report")
                    .or_else(|| outputs.first().cloned())
                    .unwrap_or_else(|| PathBuf::from("screen_report.tsv"));
                let (contamination_rate, contamination_summary) =
                    parse_screen_report(&report_path)?;
                serde_json::json!({
                    "reads_in": input.reads,
                    "reads_out": output.reads,
                    "bases_in": input.bases,
                    "bases_out": output.bases,
                    "pairs_in": pairs_in,
                    "pairs_out": pairs_out,
                    "contamination_rate": contamination_rate,
                    "contamination_summary": contamination_summary,
                })
            }
        }
        "fastq.deplete_rrna" => {
            let report_path = path_from_params(&plan.params, "report_json")
                .or_else(|| {
                    plan.io
                        .outputs
                        .iter()
                        .find(|artifact| artifact.name.as_str() == "rrna_report_json")
                        .map(|artifact| artifact.path.clone())
                })
                .or_else(|| {
                    let fallback = plan.out_dir.join("rrna_report.json");
                    fallback.exists().then_some(fallback)
                });
            let governed_report = report_path
                .and_then(|path| std::fs::read_to_string(&path).ok())
                .and_then(|raw| crate::observer::parse_deplete_rrna_report(&raw).ok());
            if let Some(report) = governed_report {
                serde_json::json!({
                    "reads_in": report.reads_in,
                    "reads_out": report.reads_out,
                    "reads_removed": report.reads_removed,
                    "bases_in": report.bases_in,
                    "bases_out": report.bases_out,
                    "bases_removed": report.bases_removed,
                    "pairs_in": report.pairs_in,
                    "pairs_out": report.pairs_out,
                    "rrna_fraction_removed": report.rrna_fraction_removed,
                    "database_artifact_id": report.database_artifact_id,
                    "screening_engine": report.screening_engine,
                    "report_format": report.report_format,
                    "paired_mode": report.paired_mode,
                })
            } else {
                let stats = stats_for_paths(&[
                    inputs.first().map(PathBuf::as_path),
                    outputs.first().map(PathBuf::as_path),
                ])?;
                let input = stats.first().copied().unwrap_or_else(zero_seqkit_metrics);
                let output = stats.get(1).copied().unwrap_or_else(zero_seqkit_metrics);
                let (pairs_in, pairs_out) = pair_counts_from_paths(inputs, outputs)?;
                let reads_removed = input.reads.saturating_sub(output.reads);
                let bases_removed = input.bases.saturating_sub(output.bases);
                let rrna_fraction_removed = if input.reads == 0 {
                    0.0
                } else {
                    reads_removed as f64 / input.reads as f64
                };
                serde_json::json!({
                    "reads_in": input.reads,
                    "reads_out": output.reads,
                    "reads_removed": reads_removed,
                    "bases_in": input.bases,
                    "bases_out": output.bases,
                    "bases_removed": bases_removed,
                    "pairs_in": pairs_in,
                    "pairs_out": pairs_out,
                    "rrna_fraction_removed": rrna_fraction_removed,
                })
            }
        }
        "fastq.deplete_reference_contaminants" => {
            let report_path = path_from_params(&plan.params, "report_json")
                .or_else(|| {
                    plan.io
                        .outputs
                        .iter()
                        .find(|artifact| artifact.name.as_str() == "contaminant_screen_report_json")
                        .map(|artifact| artifact.path.clone())
                })
                .or_else(|| {
                    let fallback = plan.out_dir.join("contaminant_screen_report.json");
                    fallback.exists().then_some(fallback)
                });
            let governed_report = report_path
                .and_then(|path| std::fs::read_to_string(&path).ok())
                .and_then(|raw| {
                    crate::observer::parse_deplete_reference_contaminants_report(&raw).ok()
                });
            if let Some(report) = governed_report {
                serde_json::json!({
                    "reads_in": report.reads_in,
                    "reads_out": report.reads_out,
                    "reads_removed": report.reads_removed,
                    "bases_in": report.bases_in,
                    "bases_out": report.bases_out,
                    "bases_removed": report.bases_removed,
                    "pairs_in": report.pairs_in,
                    "pairs_out": report.pairs_out,
                    "contaminant_fraction_removed": report.contaminant_fraction_removed,
                    "reference_catalog_id": report.reference_catalog_id,
                    "contaminant_reference": report.contaminant_reference,
                    "reference_index_backend": report.reference_index_backend,
                    "paired_mode": report.paired_mode,
                })
            } else {
                let stats = stats_for_paths(&[
                    inputs.first().map(PathBuf::as_path),
                    outputs.first().map(PathBuf::as_path),
                ])?;
                let input = stats.first().copied().unwrap_or_else(zero_seqkit_metrics);
                let output = stats.get(1).copied().unwrap_or_else(zero_seqkit_metrics);
                let (pairs_in, pairs_out) = pair_counts_from_paths(inputs, outputs)?;
                let reads_removed = input.reads.saturating_sub(output.reads);
                let bases_removed = input.bases.saturating_sub(output.bases);
                let contaminant_fraction_removed = if input.reads == 0 {
                    0.0
                } else {
                    reads_removed as f64 / input.reads as f64
                };
                serde_json::json!({
                    "reads_in": input.reads,
                    "reads_out": output.reads,
                    "reads_removed": reads_removed,
                    "bases_in": input.bases,
                    "bases_out": output.bases,
                    "bases_removed": bases_removed,
                    "pairs_in": pairs_in,
                    "pairs_out": pairs_out,
                    "contaminant_fraction_removed": contaminant_fraction_removed,
                })
            }
        }
        "fastq.deplete_host" => {
            let report_path = path_from_params(&plan.params, "report_json")
                .or_else(|| {
                    plan.io
                        .outputs
                        .iter()
                        .find(|artifact| artifact.name.as_str() == "host_depletion_report_json")
                        .map(|artifact| artifact.path.clone())
                })
                .or_else(|| {
                    let fallback = plan.out_dir.join("host_depletion_report.json");
                    fallback.exists().then_some(fallback)
                });
            let governed_report = report_path
                .and_then(|path| std::fs::read_to_string(&path).ok())
                .and_then(|raw| crate::observer::parse_deplete_host_report(&raw).ok());
            if let Some(report) = governed_report {
                serde_json::json!({
                    "reads_in": report.reads_in,
                    "reads_out": report.reads_out,
                    "reads_removed": report.reads_removed,
                    "bases_in": report.bases_in,
                    "bases_out": report.bases_out,
                    "bases_removed": report.bases_removed,
                    "pairs_in": report.pairs_in,
                    "pairs_out": report.pairs_out,
                    "host_fraction_removed": report.host_fraction_removed,
                    "reference_scope": report.reference_scope,
                    "reference_catalog_id": report.reference_catalog_id,
                    "reference_index_artifact_id": report.reference_index_artifact_id,
                    "reference_index_backend": report.reference_index_backend,
                    "identity_threshold": report.identity_threshold,
                    "paired_mode": report.paired_mode,
                })
            } else {
                let stats = stats_for_paths(&[
                    inputs.first().map(PathBuf::as_path),
                    outputs.first().map(PathBuf::as_path),
                ])?;
                let input = stats.first().copied().unwrap_or_else(zero_seqkit_metrics);
                let output = stats.get(1).copied().unwrap_or_else(zero_seqkit_metrics);
                let (pairs_in, pairs_out) = pair_counts_from_paths(inputs, outputs)?;
                let reads_removed = input.reads.saturating_sub(output.reads);
                let bases_removed = input.bases.saturating_sub(output.bases);
                let host_fraction_removed = if input.reads == 0 {
                    0.0
                } else {
                    reads_removed as f64 / input.reads as f64
                };
                serde_json::json!({
                    "reads_in": input.reads,
                    "reads_out": output.reads,
                    "reads_removed": reads_removed,
                    "bases_in": input.bases,
                    "bases_out": output.bases,
                    "bases_removed": bases_removed,
                    "pairs_in": pairs_in,
                    "pairs_out": pairs_out,
                    "host_fraction_removed": host_fraction_removed,
                })
            }
        }
        _ => serde_json::json!({}),
    };
    if plan.stage_id.0.starts_with(id_catalog::FASTQ_PREFIX) {
        if let Some(obj) = metrics.as_object_mut() {
            if !obj.contains_key("pairs_in") || !obj.contains_key("pairs_out") {
                let (pairs_in, pairs_out) = pair_counts_from_paths(inputs, outputs)?;
                if !obj.contains_key("pairs_in") {
                    obj.insert("pairs_in".to_string(), serde_json::to_value(pairs_in)?);
                }
                if !obj.contains_key("pairs_out") {
                    obj.insert("pairs_out".to_string(), serde_json::to_value(pairs_out)?);
                }
            }
        }
    }
    Ok(metrics)
}
