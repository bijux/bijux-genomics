use std::path::PathBuf;

use anyhow::Result;
use bijux_dna_core::id_catalog;
use bijux_dna_stage_contract::StagePlanV1;

use super::{analysis_feature_tables, analysis_screening};

pub(super) fn stage_metrics_for_stage(
    plan: &StagePlanV1,
    inputs: &[PathBuf],
    outputs: &[PathBuf],
) -> Option<Result<serde_json::Value>> {
    match plan.stage_id.as_str() {
        "fastq.normalize_abundance" => {
            Some(Ok(analysis_feature_tables::normalize_abundance_metrics(plan)))
        }
        "fastq.infer_asvs" => Some(Ok(analysis_feature_tables::infer_asvs_metrics(plan))),
        "fastq.cluster_otus" => Some(Ok(analysis_feature_tables::cluster_otus_metrics(plan))),
        "fastq.index_reference" => Some(Ok(analysis_feature_tables::index_reference_metrics(plan))),
        id_catalog::FASTQ_SCREEN => Some(analysis_screening::screen_metrics(plan, inputs, outputs)),
        "fastq.deplete_rrna" => {
            Some(analysis_screening::deplete_rrna_metrics(plan, inputs, outputs))
        }
        "fastq.deplete_reference_contaminants" => {
            Some(analysis_screening::deplete_reference_contaminants_metrics(plan, inputs, outputs))
        }
        "fastq.deplete_host" => {
            Some(analysis_screening::deplete_host_metrics(plan, inputs, outputs))
        }
        _ => None,
    }
}
