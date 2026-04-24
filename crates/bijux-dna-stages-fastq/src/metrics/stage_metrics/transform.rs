use std::path::PathBuf;

use anyhow::Result;
use bijux_dna_core::id_catalog;
use bijux_dna_stage_contract::StagePlanV1;

use super::{transform_filtering, transform_pairing};

pub(super) fn stage_metrics_for_stage(
    plan: &StagePlanV1,
    inputs: &[PathBuf],
    outputs: &[PathBuf],
) -> Option<Result<serde_json::Value>> {
    match plan.stage_id.as_str() {
        id_catalog::FASTQ_TRIM => Some(transform_filtering::trim_metrics(plan, inputs, outputs)),
        id_catalog::FASTQ_FILTER => {
            Some(transform_filtering::filter_metrics(plan, inputs, outputs))
        }
        id_catalog::FASTQ_DEDUPLICATE => {
            Some(transform_pairing::deduplicate_metrics(plan, inputs, outputs))
        }
        id_catalog::FASTQ_LOW_COMPLEXITY => {
            Some(transform_filtering::low_complexity_metrics(plan, inputs, outputs))
        }
        id_catalog::FASTQ_MERGE => Some(transform_pairing::merge_metrics(plan, inputs, outputs)),
        id_catalog::FASTQ_VALIDATE_PRE => Some(transform_pairing::validate_metrics(plan, inputs)),
        _ => None,
    }
}
