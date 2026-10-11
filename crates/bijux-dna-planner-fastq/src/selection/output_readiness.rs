//! Declaration closure and report selection for governed FASTQ stage/tool outputs.
use super::FastqDomainToolStageOutputContract;
use anyhow::{ensure, Context, Result};
use bijux_dna_domain_fastq::stages::ports::stage_output_ids_in_manifest_order;
use std::collections::BTreeSet;

/// Declared output readiness, without executing tools or validating report payloads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FastqOutputDeclarationAssessment {
    pub normalized_metrics_output_id: Option<String>,
    pub raw_output_artifact_ids: Vec<String>,
    pub missing_declarations: Vec<String>,
}

impl FastqOutputDeclarationAssessment {
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.missing_declarations.is_empty()
    }
}

/// Select a recognized report shared by the stage and tool, and check required declarations.
/// File suffixes and stage-only defaults cannot establish a tool's report contract.
///
/// # Errors
/// Rejects unknown stages and duplicate artifact declarations.
pub fn assess_fastq_output_declarations(
    contract: &FastqDomainToolStageOutputContract,
) -> Result<FastqOutputDeclarationAssessment> {
    let stage_outputs = stage_output_ids_in_manifest_order(contract.stage_id.as_str())
        .context("unknown FASTQ stage output contract")?;
    for (label, values) in [
        ("tool.outputs", &contract.declared_output_ids),
        ("execution_contract.expected_outputs", &contract.execution_expected_output_ids),
        ("stage_contract.expected_artifacts", &contract.stage_expected_artifact_ids),
    ] {
        ensure!(
            values.iter().collect::<BTreeSet<_>>().len() == values.len(),
            "{label} repeats an artifact"
        );
    }
    let normalized = select_report(&stage_outputs, &contract.stage_expected_artifact_ids);
    let declared = contract.declared_output_ids.iter().collect::<BTreeSet<_>>();
    let execution = contract.execution_expected_output_ids.iter().collect::<BTreeSet<_>>();
    let mut missing = Vec::new();
    if contract.stage_expected_artifact_ids.is_empty() {
        missing.push("stage_contract.expected_artifacts".to_owned());
    }
    for id in &contract.stage_expected_artifact_ids {
        if !declared.contains(id) {
            missing.push(format!("tool.outputs:{id}"));
        }
        if !execution.contains(id) {
            missing.push(format!("execution_contract.expected_outputs:{id}"));
        }
    }
    if normalized.is_none() {
        missing.push("normalized_metrics_output".to_owned());
    }
    let mut raw_outputs = contract
        .stage_expected_artifact_ids
        .iter()
        .filter(|id| Some(id.as_str()) != normalized.as_deref())
        .cloned()
        .collect::<Vec<_>>();
    // Report-only bindings retain the public dual-role declaration; this is not raw-output proof.
    if raw_outputs.is_empty() {
        raw_outputs.extend(normalized.iter().cloned());
    }
    if raw_outputs.is_empty() {
        missing.push("raw_output".to_owned());
    }
    Ok(FastqOutputDeclarationAssessment {
        raw_output_artifact_ids: raw_outputs,
        normalized_metrics_output_id: normalized,
        missing_declarations: missing,
    })
}

fn select_report(stage_outputs: &[String], expected: &[String]) -> Option<String> {
    [
        "duplicate_signal_report",
        "library_complexity_report",
        "validation_report",
        "classification_report_json",
        "host_depletion_report_json",
        "rrna_report_json",
        "contaminant_screen_report_json",
        "filter_report_json",
        "qc_json",
        "report_json",
    ]
    .into_iter()
    .find(|candidate| {
        stage_outputs.iter().any(|id| id == candidate) && expected.iter().any(|id| id == candidate)
    })
    .map(str::to_owned)
}

#[cfg(test)]
#[path = "../../tests/contracts/selection/output_readiness.rs"]
mod tests;
