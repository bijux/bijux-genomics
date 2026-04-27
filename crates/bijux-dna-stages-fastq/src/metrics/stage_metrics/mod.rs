mod analysis;
mod analysis_feature_tables;
mod analysis_screening;
mod reporting;
mod transform;
mod transform_filtering;
mod transform_pairing;

use super::envelope_support::pair_counts_from_paths;
use analysis::stage_metrics_for_stage as analysis_stage_metrics_for_stage;
use anyhow::Result;
use bijux_dna_core::id_catalog;
use bijux_dna_stage_contract::StagePlanV1;
use reporting::stage_metrics_for_stage as reporting_stage_metrics_for_stage;
use std::path::PathBuf;
use transform::stage_metrics_for_stage as transform_stage_metrics_for_stage;

pub(super) fn stage_metrics_for_plan(
    plan: &StagePlanV1,
    inputs: &[PathBuf],
    outputs: &[PathBuf],
) -> Result<serde_json::Value> {
    let metrics = match plan.stage_id.as_str() {
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
        "fastq.normalize_abundance"
        | "fastq.infer_asvs"
        | "fastq.cluster_otus"
        | "fastq.index_reference"
        | id_catalog::FASTQ_SCREEN
        | "fastq.deplete_rrna"
        | "fastq.deplete_reference_contaminants"
        | "fastq.deplete_host" => match analysis_stage_metrics_for_stage(plan, inputs, outputs) {
            Some(metrics) => metrics,
            None => unreachable!("analysis stage ids must be handled"),
        },
        _ => Ok(serde_json::json!({})),
    };
    let mut metrics = metrics?;
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
