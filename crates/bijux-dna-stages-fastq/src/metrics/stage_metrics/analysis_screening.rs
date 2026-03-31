use std::path::PathBuf;

use anyhow::Result;
use bijux_dna_stage_contract::StagePlanV1;

use crate::metrics::envelope_support::{
    pair_counts_from_paths, path_from_params, stats_for_paths, zero_seqkit_metrics,
};
use crate::metrics::filters::parse_screen_report;

pub(super) fn screen_metrics(
    plan: &StagePlanV1,
    inputs: &[PathBuf],
    outputs: &[PathBuf],
) -> Result<serde_json::Value> {
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
    Ok(if let Some(report) = governed_report {
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
        let (contamination_rate, contamination_summary) = parse_screen_report(&report_path)?;
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
    })
}

pub(super) fn deplete_rrna_metrics(
    plan: &StagePlanV1,
    inputs: &[PathBuf],
    outputs: &[PathBuf],
) -> Result<serde_json::Value> {
    let (pairs_in, pairs_out) = pair_counts_from_paths(inputs, outputs)?;
    let report_path = path_from_params(&plan.params, "rrna_report_json").or_else(|| {
        let fallback = plan.out_dir.join("rrna_report.json");
        fallback.exists().then_some(fallback)
    });
    let report = report_path
        .and_then(|path| std::fs::read_to_string(path).ok())
        .and_then(|raw| crate::observer::parse_deplete_rrna_report(&raw).ok());
    Ok(if let Some(report) = report {
        serde_json::json!({
            "reads_in": report.reads_in,
            "reads_out": report.reads_out,
            "bases_in": report.bases_in,
            "bases_out": report.bases_out,
            "pairs_in": report.pairs_in.or(pairs_in),
            "pairs_out": report.pairs_out.or(pairs_out),
            "reads_removed": report.reads_removed,
            "bases_removed": report.bases_removed,
            "rrna_fraction_removed": report.rrna_fraction_removed,
            "database_artifact_id": report.database_artifact_id,
            "database_build_id": report.database_build_id,
            "raw_backend_report_format": report.raw_backend_report_format,
        })
    } else {
        serde_json::json!({})
    })
}

pub(super) fn deplete_reference_contaminants_metrics(
    plan: &StagePlanV1,
    inputs: &[PathBuf],
    outputs: &[PathBuf],
) -> Result<serde_json::Value> {
    let (pairs_in, pairs_out) = pair_counts_from_paths(inputs, outputs)?;
    let report_path =
        path_from_params(&plan.params, "contaminant_screen_report_json").or_else(|| {
            let fallback = plan.out_dir.join("contaminant_screen_report.json");
            fallback.exists().then_some(fallback)
        });
    let report = report_path
        .and_then(|path| std::fs::read_to_string(path).ok())
        .and_then(|raw| crate::observer::parse_deplete_reference_contaminants_report(&raw).ok());
    Ok(if let Some(report) = report {
        serde_json::json!({
            "reads_in": report.reads_in,
            "reads_out": report.reads_out,
            "bases_in": report.bases_in,
            "bases_out": report.bases_out,
            "pairs_in": report.pairs_in.or(pairs_in),
            "pairs_out": report.pairs_out.or(pairs_out),
            "reads_removed": report.reads_removed,
            "bases_removed": report.bases_removed,
            "contaminant_fraction_removed": report.contaminant_fraction_removed,
            "reference_catalog_id": report.reference_catalog_id,
            "reference_build_id": report.reference_build_id,
            "reference_index_backend": report.reference_index_backend,
            "raw_backend_report_format": report.raw_backend_report_format,
        })
    } else {
        serde_json::json!({})
    })
}

pub(super) fn deplete_host_metrics(
    plan: &StagePlanV1,
    inputs: &[PathBuf],
    outputs: &[PathBuf],
) -> Result<serde_json::Value> {
    let (pairs_in, pairs_out) = pair_counts_from_paths(inputs, outputs)?;
    let report_path = path_from_params(&plan.params, "host_depletion_report_json").or_else(|| {
        let fallback = plan.out_dir.join("host_depletion_report.json");
        fallback.exists().then_some(fallback)
    });
    let report = report_path
        .and_then(|path| std::fs::read_to_string(path).ok())
        .and_then(|raw| crate::observer::parse_deplete_host_report(&raw).ok());
    Ok(if let Some(report) = report {
        serde_json::json!({
            "reads_in": report.reads_in,
            "reads_out": report.reads_out,
            "bases_in": report.bases_in,
            "bases_out": report.bases_out,
            "pairs_in": report.pairs_in.or(pairs_in),
            "pairs_out": report.pairs_out.or(pairs_out),
            "reads_removed": report.reads_removed,
            "bases_removed": report.bases_removed,
            "host_fraction_removed": report.host_fraction_removed,
            "reference_catalog_id": report.reference_catalog_id,
            "reference_build_id": report.reference_build_id,
            "reference_index_backend": report.reference_index_backend,
            "identity_threshold": report.identity_threshold,
            "raw_backend_report_format": report.raw_backend_report_format,
        })
    } else {
        serde_json::json!({})
    })
}
