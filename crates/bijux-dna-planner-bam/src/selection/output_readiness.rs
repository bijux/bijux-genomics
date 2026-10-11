//! Declaration closure and report selection for governed BAM stage/tool outputs.
use super::BamDomainToolStageOutputContract;
use anyhow::{ensure, Result};
use bijux_dna_domain_bam::stage_specs::stage_declared_output_ids;
use std::collections::BTreeSet;

/// Declared output readiness, without executing tools or validating report payloads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BamOutputDeclarationAssessment {
    pub normalized_metrics_output_id: Option<String>,
    pub raw_output_artifact_ids: Vec<String>,
    pub missing_declarations: Vec<String>,
}

impl BamOutputDeclarationAssessment {
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
pub fn assess_bam_output_declarations(
    contract: &BamDomainToolStageOutputContract,
) -> Result<BamOutputDeclarationAssessment> {
    let stage_outputs = stage_declared_output_ids(contract.stage_id.as_str())?;
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
    let normalized = select_report(stage_outputs, &contract.stage_expected_artifact_ids);
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
    Ok(BamOutputDeclarationAssessment {
        raw_output_artifact_ids: contract
            .stage_expected_artifact_ids
            .iter()
            .filter(|id| Some(id.as_str()) != normalized.as_deref())
            .cloned()
            .collect(),
        normalized_metrics_output_id: normalized,
        missing_declarations: missing,
    })
}

fn select_report(stage_outputs: &[String], expected: &[String]) -> Option<String> {
    [
        "validation_report",
        "damage_report",
        "authenticity_report",
        "contamination_report",
        "sex_report",
        "bias_report",
        "genotyping_report",
        "kinship_report",
        "haplogroups",
        "coverage_summary",
        "endogenous_report",
        "duplication_report",
        "insert_size_report",
        "gc_bias_report",
        "align_metrics",
        "recal_report",
        "report_json",
        "summary",
        "stage_metrics",
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
