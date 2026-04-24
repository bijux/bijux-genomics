//! Owner: bijux-dna-bench
//! Fairness and input-consistency checks for benchmark summarization.

use std::collections::{BTreeMap, BTreeSet};

use bijux_dna_bench_model::contract::validate_observation;
use bijux_dna_bench_model::{BenchError, BenchmarkObservation, BenchmarkSuiteSpec};

use super::summary_scope::{stage_scope_label, StageDatasetScope, StageDatasetToolScope};

pub(super) struct SummaryFairnessOutcome {
    pub scientifically_invalid: bool,
    pub invalid_reasons: Vec<String>,
    pub warnings: Vec<String>,
}

pub(super) fn evaluate_summary_fairness(
    suite: &BenchmarkSuiteSpec,
    observations: &[BenchmarkObservation],
) -> Result<SummaryFairnessOutcome, BenchError> {
    let mut scientifically_invalid = false;
    let mut invalid_reasons = Vec::new();
    for obs in observations {
        if let Err(err) = validate_observation(obs) {
            match err {
                BenchError::MissingConfounder { field } => {
                    scientifically_invalid = true;
                    invalid_reasons.push(format!("missing_confounder:{field}"));
                }
                BenchError::InvalidObservation { reason } => {
                    scientifically_invalid = true;
                    invalid_reasons.push(format!("invalid_observation:{reason}"));
                }
                other => return Err(other),
            }
        }
    }

    let mut warnings = Vec::new();
    if suite.replicate_policy.count < 3 {
        warnings.push("low_power".to_string());
    }

    let mut stage_dataset_inputs: BTreeMap<StageDatasetScope, BTreeSet<String>> = BTreeMap::new();
    let mut stage_dataset_tool_params: BTreeMap<StageDatasetToolScope, BTreeSet<String>> =
        BTreeMap::new();
    for obs in observations {
        stage_dataset_inputs
            .entry((
                obs.stage_id.clone(),
                obs.dataset_id.clone(),
                obs.stage_instance_id.clone(),
                obs.lineage_id.clone(),
            ))
            .or_default()
            .insert(obs.input_hash.clone());
        stage_dataset_tool_params
            .entry((
                obs.stage_id.clone(),
                obs.dataset_id.clone(),
                obs.stage_instance_id.clone(),
                obs.lineage_id.clone(),
                obs.tool_id.clone(),
            ))
            .or_default()
            .insert(obs.params_hash.clone());
    }

    for ((stage_id, dataset_id, stage_instance_id, lineage_id), hashes) in &stage_dataset_inputs {
        if hashes.len() > 1 {
            scientifically_invalid = true;
            let warning = format!(
                "fairness_input_mismatch:{}",
                stage_scope_label(
                    stage_id,
                    stage_instance_id.as_deref(),
                    lineage_id.as_deref(),
                    dataset_id
                )
            );
            warnings.push(warning.clone());
            invalid_reasons.push(warning);
        }
    }

    for ((stage_id, dataset_id, stage_instance_id, lineage_id, tool_id), hashes) in
        &stage_dataset_tool_params
    {
        if hashes.len() > 1 {
            scientifically_invalid = true;
            let warning = format!(
                "fairness_param_hash_mismatch:{}:{tool}",
                stage_scope_label(
                    stage_id,
                    stage_instance_id.as_deref(),
                    lineage_id.as_deref(),
                    dataset_id
                ),
                tool = tool_id
            );
            warnings.push(warning.clone());
            invalid_reasons.push(warning);
        }
    }

    Ok(SummaryFairnessOutcome { scientifically_invalid, invalid_reasons, warnings })
}
